//! Converts registry `packages` / `remotes` into [`InstallOption`]s.
//!
//! Registry schema 2025-07-09 uses snake_case and later versions camelCase, so every
//! multi-word field accepts both spellings.

use super::{InstallInput, InstallOption, PackageType};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawPackage {
    #[serde(default, alias = "registry_type")]
    pub registry_type: Option<String>,
    #[serde(default, alias = "registry_base_url")]
    pub registry_base_url: Option<String>,
    #[serde(default)]
    pub identifier: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default, alias = "runtime_hint")]
    pub runtime_hint: Option<String>,
    #[serde(default, alias = "runtime_arguments")]
    pub runtime_arguments: Vec<RawArgument>,
    #[serde(default, alias = "package_arguments")]
    pub package_arguments: Vec<RawArgument>,
    #[serde(default, alias = "environment_variables")]
    pub environment_variables: Vec<RawKeyValue>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawRemote {
    #[serde(default, rename = "type", alias = "transport_type")]
    pub transport: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub headers: Vec<RawKeyValue>,
    #[serde(default)]
    pub variables: BTreeMap<String, RawVariable>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawArgument {
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default, alias = "value_hint")]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default, alias = "is_required")]
    pub is_required: bool,
    #[serde(default, alias = "is_secret")]
    pub is_secret: bool,
    #[serde(default)]
    pub variables: BTreeMap<String, RawVariable>,
}

/// An environment variable or HTTP header.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawKeyValue {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, alias = "is_required")]
    pub is_required: bool,
    #[serde(default, alias = "is_secret")]
    pub is_secret: bool,
    #[serde(default)]
    pub variables: BTreeMap<String, RawVariable>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawVariable {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default, alias = "is_required")]
    pub is_required: bool,
    #[serde(default, alias = "is_secret")]
    pub is_secret: bool,
}

pub(super) fn install_options(
    packages: &[RawPackage],
    remotes: &[RawRemote],
) -> Vec<InstallOption> {
    packages
        .iter()
        .map(package_option)
        .chain(
            remotes
                .iter()
                .filter(|remote| !remote.url.is_empty())
                .map(remote_option),
        )
        .collect()
}

/// What a package resolves to: a type we can run, or the raw type name we cannot.
enum Resolved {
    Runnable {
        package_type: PackageType,
        inferred: bool,
    },
    Unsupported(String),
}

fn resolve_package_type(package: &RawPackage) -> Resolved {
    let runnable = |package_type, inferred| Resolved::Runnable {
        package_type,
        inferred,
    };

    if let Some(declared) = package.registry_type.as_deref() {
        return match declared {
            "npm" => runnable(PackageType::Npm, false),
            "pypi" => runnable(PackageType::Pypi, false),
            "oci" => runnable(PackageType::Oci, false),
            other => Resolved::Unsupported(other.to_string()),
        };
    }

    match package.runtime_hint.as_deref() {
        Some("npx") => return runnable(PackageType::Npm, false),
        Some("uvx") => return runnable(PackageType::Pypi, false),
        Some("docker") => return runnable(PackageType::Oci, false),
        Some(_) => return runnable(PackageType::Other, true),
        None => {}
    }

    let base_url = package.registry_base_url.as_deref().unwrap_or_default();
    if base_url.contains("npmjs") {
        return runnable(PackageType::Npm, false);
    }
    if base_url.contains("pypi") {
        return runnable(PackageType::Pypi, false);
    }
    if base_url.contains("nuget") {
        return Resolved::Unsupported("nuget".to_string());
    }

    let identifier = package.identifier.as_str();
    if identifier.starts_with("http://") || identifier.starts_with("https://") {
        return Resolved::Unsupported("mcpb".to_string());
    }
    if identifier.starts_with('@') {
        return runnable(PackageType::Npm, false);
    }
    if looks_like_image(identifier) {
        return runnable(PackageType::Oci, true);
    }
    // NuGet ids are dotted PascalCase ("Azure.Mcp"); npm and PyPI names are lowercase.
    if identifier.contains('.') && identifier.chars().any(|c| c.is_ascii_uppercase()) {
        return Resolved::Unsupported("nuget".to_string());
    }
    // A bare lowercase name: npm is by far the most common publisher choice.
    runnable(PackageType::Npm, true)
}

/// `ghcr.io/org/image`, `docker.io/org/image:tag`: a registry host with a dot, then a path.
fn looks_like_image(identifier: &str) -> bool {
    identifier
        .split_once('/')
        .is_some_and(|(host, rest)| host.contains('.') && !rest.is_empty())
}

fn pinned_version(package: &RawPackage) -> Option<&str> {
    package
        .version
        .as_deref()
        .filter(|version| !version.is_empty() && *version != "latest")
}

fn package_option(package: &RawPackage) -> InstallOption {
    let (package_type, inferred) = match resolve_package_type(package) {
        Resolved::Runnable {
            package_type,
            inferred,
        } => (package_type, inferred),
        Resolved::Unsupported(package_type) => {
            return InstallOption::Unsupported {
                package_type,
                identifier: package.identifier.clone(),
            }
        }
    };

    let mut inputs = Inputs::default();
    let runtime_groups = argument_groups(&package.runtime_arguments, &mut inputs);
    let package_groups = argument_groups(&package.package_arguments, &mut inputs);
    let env = key_values(&package.environment_variables, &mut inputs);
    let version = pinned_version(package);

    let mut arg_groups = Vec::new();
    let program = match package_type {
        PackageType::Npm => {
            arg_groups.push(vec!["-y".to_string()]);
            arg_groups.extend(runtime_groups);
            arg_groups.push(vec![match version {
                Some(version) => format!("{}@{version}", package.identifier),
                None => package.identifier.clone(),
            }]);
            "npx".to_string()
        }
        PackageType::Pypi => {
            arg_groups.extend(runtime_groups);
            arg_groups.push(vec![match version {
                Some(version) => format!("{}=={version}", package.identifier),
                None => package.identifier.clone(),
            }]);
            "uvx".to_string()
        }
        PackageType::Oci => {
            arg_groups.push(vec![
                "run".to_string(),
                "-i".to_string(),
                "--rm".to_string(),
            ]);
            arg_groups.extend(runtime_groups);
            // `-e NAME` forwards the variable from the env the MCP client passes to docker.
            arg_groups.extend(env.keys().map(|name| vec!["-e".to_string(), name.clone()]));
            let has_tag = package
                .identifier
                .rsplit('/')
                .next()
                .is_some_and(|last| last.contains(':') || last.contains('@'));
            arg_groups.push(vec![match version {
                Some(version) if !has_tag => format!("{}:{version}", package.identifier),
                _ => package.identifier.clone(),
            }]);
            "docker".to_string()
        }
        PackageType::Other => {
            // The publisher named a runtime we do not model; trust its arguments as given.
            arg_groups.extend(runtime_groups);
            package
                .runtime_hint
                .clone()
                .unwrap_or_else(|| package.identifier.clone())
        }
    };
    arg_groups.extend(package_groups);

    InstallOption::Stdio {
        package_type,
        identifier: package.identifier.clone(),
        program,
        arg_groups,
        env,
        inputs: inputs.into_vec(),
        inferred,
    }
}

fn remote_option(remote: &RawRemote) -> InstallOption {
    let mut inputs = Inputs::default();
    inputs.add_from_template(&remote.url, &remote.variables, true);
    let headers = key_values(&remote.headers, &mut inputs);
    InstallOption::Http {
        transport: if remote.transport.is_empty() {
            "streamable-http".to_string()
        } else {
            remote.transport.clone()
        },
        url: remote.url.clone(),
        headers,
        inputs: inputs.into_vec(),
    }
}

/// Environment variables or headers: fixed values are kept, everything else becomes a
/// `{placeholder}` backed by an input.
fn key_values(items: &[RawKeyValue], inputs: &mut Inputs) -> BTreeMap<String, String> {
    items
        .iter()
        .filter(|item| !item.name.is_empty())
        .map(|item| {
            let value = match item.value.as_deref().filter(|value| !value.is_empty()) {
                Some(value) => {
                    inputs.add_from_template(value, &item.variables, item.is_required);
                    value.to_string()
                }
                None => {
                    inputs.add(InstallInput {
                        key: item.name.clone(),
                        description: item.description.clone(),
                        required: item.is_required,
                        secret: item.is_secret,
                        default_value: item.default.clone(),
                    });
                    placeholder(&item.name)
                }
            };
            (item.name.clone(), value)
        })
        .collect()
}

/// Only arguments that carry a value, a default, or are required make it into the
/// command; optional flags without values are documentation, not configuration.
fn argument_groups(arguments: &[RawArgument], inputs: &mut Inputs) -> Vec<Vec<String>> {
    arguments
        .iter()
        .filter_map(|argument| {
            let fixed = argument
                .value
                .as_deref()
                .or(argument.default.as_deref())
                .filter(|value| !value.is_empty());
            let value = match fixed {
                Some(value) => {
                    inputs.add_from_template(value, &argument.variables, argument.is_required);
                    Some(value.to_string())
                }
                None if !argument.is_required => return None,
                None if argument.format.as_deref() == Some("boolean") => None,
                None => {
                    let key = argument_key(argument)?;
                    inputs.add(InstallInput {
                        key: key.clone(),
                        description: argument.description.clone(),
                        required: true,
                        secret: argument.is_secret,
                        default_value: None,
                    });
                    Some(placeholder(&key))
                }
            };

            let group = match (argument.kind.as_str(), argument.name.as_deref(), value) {
                ("named", Some(name), Some(value)) => vec![name.to_string(), value],
                ("named", Some(name), None) => vec![name.to_string()],
                (_, _, Some(value)) => vec![value],
                _ => return None,
            };
            Some(group)
        })
        .collect()
}

fn argument_key(argument: &RawArgument) -> Option<String> {
    let raw = match argument.kind.as_str() {
        "named" => argument.name.as_deref(),
        _ => argument.value_hint.as_deref().or(argument.name.as_deref()),
    }?;
    let key = raw.trim_start_matches('-');
    (!key.is_empty()).then(|| key.to_string())
}

fn placeholder(key: &str) -> String {
    format!("{{{key}}}")
}

/// Placeholder names in `{name}` form, in order of appearance.
fn template_keys(value: &str) -> Vec<&str> {
    let mut keys = Vec::new();
    let mut rest = value;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else { break };
        let key = &after[..end];
        // Only identifier-like names; `{"a":1}` in a value is JSON, not a placeholder.
        if !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        {
            keys.push(key);
        }
        rest = &after[end + 1..];
    }
    keys
}

/// Inputs collected while converting one install option, de-duplicated by key.
#[derive(Default)]
struct Inputs(Vec<InstallInput>);

impl Inputs {
    fn add(&mut self, input: InstallInput) {
        match self.0.iter_mut().find(|existing| existing.key == input.key) {
            Some(existing) => {
                existing.required |= input.required;
                existing.secret |= input.secret;
                if existing.description.is_none() {
                    existing.description = input.description;
                }
            }
            None => self.0.push(input),
        }
    }

    /// Adds an input for every `{placeholder}` in `value`, described by `variables`
    /// when the publisher declared them.
    fn add_from_template(
        &mut self,
        value: &str,
        variables: &BTreeMap<String, RawVariable>,
        parent_required: bool,
    ) {
        for key in template_keys(value) {
            let variable = variables.get(key);
            self.add(InstallInput {
                key: key.to_string(),
                description: variable.and_then(|v| v.description.clone()),
                required: variable.map_or(parent_required, |v| v.is_required),
                secret: variable.is_some_and(|v| v.is_secret),
                default_value: variable.and_then(|v| v.default.clone()),
            });
        }
    }

    fn into_vec(self) -> Vec<InstallInput> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn packages(value: serde_json::Value) -> Vec<RawPackage> {
        serde_json::from_value(value).unwrap()
    }

    fn remotes(value: serde_json::Value) -> Vec<RawRemote> {
        serde_json::from_value(value).unwrap()
    }

    fn flat_args(option: &InstallOption) -> Vec<String> {
        match option {
            InstallOption::Stdio { arg_groups, .. } => arg_groups.concat(),
            other => panic!("expected stdio, got {other:?}"),
        }
    }

    fn input(key: &str, required: bool, secret: bool) -> InstallInput {
        InstallInput {
            key: key.to_string(),
            description: None,
            required,
            secret,
            default_value: None,
        }
    }

    #[test]
    fn npm_package_runs_through_npx_with_pinned_version_and_secret_env_input() {
        let options = install_options(
            &packages(json!([{
                "registryType": "npm",
                "identifier": "@upstash/context7-mcp",
                "version": "4.1.1",
                "environmentVariables": [{ "name": "CONTEXT7_API_KEY", "description": "API key", "isSecret": true }]
            }])),
            &[],
        );

        let InstallOption::Stdio {
            package_type,
            program,
            env,
            inputs,
            inferred,
            ..
        } = &options[0]
        else {
            panic!("expected stdio");
        };
        assert_eq!(*package_type, PackageType::Npm);
        assert_eq!(program, "npx");
        assert_eq!(
            flat_args(&options[0]),
            ["-y", "@upstash/context7-mcp@4.1.1"]
        );
        assert_eq!(
            env.get("CONTEXT7_API_KEY").map(String::as_str),
            Some("{CONTEXT7_API_KEY}")
        );
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].key, "CONTEXT7_API_KEY");
        assert!(inputs[0].secret);
        assert_eq!(inputs[0].description.as_deref(), Some("API key"));
        assert!(!inferred);
    }

    #[test]
    fn pypi_package_runs_through_uvx() {
        let options = install_options(
            &packages(
                json!([{ "registry_type": "pypi", "identifier": "docker-mcp-server", "version": "2.1.1" }]),
            ),
            &[],
        );
        let InstallOption::Stdio { program, .. } = &options[0] else {
            panic!()
        };
        assert_eq!(program, "uvx");
        assert_eq!(flat_args(&options[0]), ["docker-mcp-server==2.1.1"]);
    }

    #[test]
    fn latest_version_is_not_pinned() {
        let options = install_options(
            &packages(
                json!([{ "runtime_hint": "uvx", "identifier": "serena", "version": "latest" }]),
            ),
            &[],
        );
        assert_eq!(flat_args(&options[0]), ["serena"]);
    }

    #[test]
    fn oci_image_runs_through_docker_and_forwards_env_names() {
        let options = install_options(
            &packages(json!([{
                "registryType": "oci",
                "identifier": "ghcr.io/example/server",
                "version": "1.2.0",
                "environmentVariables": [{ "name": "API_TOKEN", "isRequired": true, "isSecret": true }]
            }])),
            &[],
        );
        let InstallOption::Stdio {
            program,
            env,
            inputs,
            ..
        } = &options[0]
        else {
            panic!()
        };
        assert_eq!(program, "docker");
        assert_eq!(
            flat_args(&options[0]),
            [
                "run",
                "-i",
                "--rm",
                "-e",
                "API_TOKEN",
                "ghcr.io/example/server:1.2.0"
            ]
        );
        assert_eq!(
            env.get("API_TOKEN").map(String::as_str),
            Some("{API_TOKEN}")
        );
        assert_eq!(inputs, &[input("API_TOKEN", true, true)]);
    }

    #[test]
    fn oci_image_with_tag_keeps_its_tag() {
        let options = install_options(
            &packages(json!([{ "identifier": "ghcr.io/github/github-mcp-server:1.12.2" }])),
            &[],
        );
        let InstallOption::Stdio {
            package_type,
            inferred,
            ..
        } = &options[0]
        else {
            panic!()
        };
        assert_eq!(*package_type, PackageType::Oci);
        assert!(inferred);
        assert_eq!(
            flat_args(&options[0]),
            [
                "run",
                "-i",
                "--rm",
                "ghcr.io/github/github-mcp-server:1.12.2"
            ]
        );
    }

    #[test]
    fn runtime_argument_templates_become_optional_inputs_in_their_own_group() {
        let options = install_options(
            &packages(json!([{
                "identifier": "ghcr.io/github/github-mcp-server:1.12.2",
                "runtime_hint": "docker",
                "runtime_arguments": [
                    { "type": "named", "name": "-p", "value": "127.0.0.1:8085:8085" },
                    {
                        "type": "named",
                        "name": "-e",
                        "value": "GITHUB_PERSONAL_ACCESS_TOKEN={token}",
                        "variables": { "token": { "description": "PAT", "is_secret": true } }
                    }
                ]
            }])),
            &[],
        );
        let InstallOption::Stdio {
            arg_groups, inputs, ..
        } = &options[0]
        else {
            panic!()
        };
        assert_eq!(
            arg_groups,
            &vec![
                vec!["run".to_string(), "-i".to_string(), "--rm".to_string()],
                vec!["-p".to_string(), "127.0.0.1:8085:8085".to_string()],
                vec![
                    "-e".to_string(),
                    "GITHUB_PERSONAL_ACCESS_TOKEN={token}".to_string()
                ],
                vec!["ghcr.io/github/github-mcp-server:1.12.2".to_string()],
            ]
        );
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].key, "token");
        assert!(!inputs[0].required);
        assert!(inputs[0].secret);
    }

    #[test]
    fn optional_arguments_without_values_are_left_out() {
        let options = install_options(
            &packages(json!([{
                "identifier": "mongodb-mcp-server",
                "version": "2.1.0",
                "registryType": "npm",
                "packageArguments": [
                    { "type": "named", "name": "--apiClientId", "description": "Atlas client id" },
                    { "type": "named", "name": "--readOnly", "format": "boolean", "isRequired": true },
                    { "type": "named", "name": "--connectionString", "isRequired": true, "isSecret": true },
                    { "type": "positional", "valueHint": "directory", "isRequired": true }
                ]
            }])),
            &[],
        );
        let InstallOption::Stdio { inputs, .. } = &options[0] else {
            panic!()
        };
        assert_eq!(
            flat_args(&options[0]),
            [
                "-y",
                "mongodb-mcp-server@2.1.0",
                "--readOnly",
                "--connectionString",
                "{connectionString}",
                "{directory}"
            ]
        );
        assert_eq!(
            inputs,
            &[
                input("connectionString", true, true),
                input("directory", true, false)
            ]
        );
    }

    #[test]
    fn fixed_env_values_are_kept_and_not_asked_for() {
        let options = install_options(
            &packages(json!([{
                "registryType": "npm",
                "identifier": "pkg",
                "environmentVariables": [
                    { "name": "MODE", "value": "stdio" },
                    { "name": "REGION", "default": "us", "description": "Region" }
                ]
            }])),
            &[],
        );
        let InstallOption::Stdio { env, inputs, .. } = &options[0] else {
            panic!()
        };
        assert_eq!(env.get("MODE").map(String::as_str), Some("stdio"));
        assert_eq!(env.get("REGION").map(String::as_str), Some("{REGION}"));
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].default_value.as_deref(), Some("us"));
    }

    #[test]
    fn package_type_is_inferred_from_hints_and_identifier_shape() {
        let cases = [
            (
                json!({ "identifier": "@scope/pkg" }),
                Some(PackageType::Npm),
            ),
            (
                json!({ "identifier": "pkg", "registry_base_url": "https://registry.npmjs.org" }),
                Some(PackageType::Npm),
            ),
            (
                json!({ "identifier": "pkg", "registry_base_url": "https://pypi.org" }),
                Some(PackageType::Pypi),
            ),
            (
                json!({ "identifier": "pkg", "runtime_hint": "npx" }),
                Some(PackageType::Npm),
            ),
            (
                json!({ "identifier": "docker.io/org/image:1.0" }),
                Some(PackageType::Oci),
            ),
            (
                json!({ "identifier": "pkg", "runtime_hint": "uv" }),
                Some(PackageType::Other),
            ),
            (
                json!({ "identifier": "tavily-mcp" }),
                Some(PackageType::Npm),
            ),
            (
                json!({ "identifier": "https://github.com/o/r/releases/download/v1/x.mcpb" }),
                None,
            ),
            (
                json!({ "identifier": "Azure.Mcp", "version": "3.0.0" }),
                None,
            ),
            (
                json!({ "identifier": "NuGet.Mcp.Server", "registry_base_url": "https://api.nuget.org/v3/index.json" }),
                None,
            ),
        ];
        for (raw, expected) in cases {
            let options = install_options(&packages(json!([raw.clone()])), &[]);
            let actual = match &options[0] {
                InstallOption::Stdio { package_type, .. } => Some(*package_type),
                InstallOption::Unsupported { .. } => None,
                other => panic!("unexpected {other:?}"),
            };
            assert_eq!(actual, expected, "for {raw}");
        }
    }

    #[test]
    fn unknown_runtime_hint_is_used_as_the_program() {
        let options = install_options(
            &packages(json!([{
                "identifier": "uv",
                "runtime_hint": "uv",
                "runtime_arguments": [{ "type": "positional", "value": "run" }]
            }])),
            &[],
        );
        let InstallOption::Stdio {
            program, inferred, ..
        } = &options[0]
        else {
            panic!()
        };
        assert_eq!(program, "uv");
        assert!(inferred);
        assert_eq!(flat_args(&options[0]), ["run"]);
    }

    #[test]
    fn declared_unsupported_types_are_reported() {
        let options = install_options(
            &packages(
                json!([{ "registryType": "mcpb", "identifier": "https://example.com/x.mcpb" }]),
            ),
            &[],
        );
        assert_eq!(
            options,
            vec![InstallOption::Unsupported {
                package_type: "mcpb".to_string(),
                identifier: "https://example.com/x.mcpb".to_string(),
            }]
        );
    }

    #[test]
    fn remote_headers_become_inputs() {
        let options = install_options(
            &[],
            &remotes(json!([{
                "transport_type": "streamable-http",
                "url": "https://app.netdata.cloud/api/v1/mcp",
                "headers": [{
                    "name": "Authorization",
                    "value": "Bearer {NETDATA_TOKEN}",
                    "is_required": true,
                    "variables": { "NETDATA_TOKEN": { "is_secret": true, "is_required": true } }
                }]
            }])),
        );
        assert_eq!(
            options,
            vec![InstallOption::Http {
                transport: "streamable-http".to_string(),
                url: "https://app.netdata.cloud/api/v1/mcp".to_string(),
                headers: BTreeMap::from([(
                    "Authorization".to_string(),
                    "Bearer {NETDATA_TOKEN}".to_string()
                )]),
                inputs: vec![input("NETDATA_TOKEN", true, true)],
            }]
        );
    }

    #[test]
    fn remote_header_without_value_is_asked_for_directly() {
        let options = install_options(
            &[],
            &remotes(json!([{
                "type": "sse",
                "url": "https://example.com/sse",
                "headers": [{ "name": "X-Api-Key", "isSecret": true }]
            }])),
        );
        let InstallOption::Http {
            headers, inputs, ..
        } = &options[0]
        else {
            panic!()
        };
        assert_eq!(
            headers.get("X-Api-Key").map(String::as_str),
            Some("{X-Api-Key}")
        );
        assert_eq!(inputs, &[input("X-Api-Key", false, true)]);
    }

    #[test]
    fn braces_that_are_not_placeholders_are_not_inputs() {
        let options = install_options(
            &packages(json!([{
                "registryType": "npm",
                "identifier": "pkg",
                "environmentVariables": [{ "name": "CONFIG", "value": "{\"mode\": 1}" }]
            }])),
            &[],
        );
        let InstallOption::Stdio { inputs, .. } = &options[0] else {
            panic!()
        };
        assert!(inputs.is_empty());
    }

    #[test]
    fn packages_come_before_remotes() {
        let options = install_options(
            &packages(json!([{ "registryType": "npm", "identifier": "pkg" }])),
            &remotes(json!([{ "type": "streamable-http", "url": "https://example.com/mcp" }])),
        );
        assert!(matches!(options[0], InstallOption::Stdio { .. }));
        assert!(matches!(options[1], InstallOption::Http { .. }));
    }
}
