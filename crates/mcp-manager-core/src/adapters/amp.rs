use crate::adapters::{managed_json_field_writes, scope_for_path, AppAdapter, ParsedSources};
use crate::core::{LocalConfigSource, MCPConfig, SupportedApp, WriteOperation};
use crate::parser::{enable_servers_for_app_at, extract_json_field_mcp_json, parse_mcp_json};
use crate::platform::PlatformContext;

pub struct AmpAdapter;

/// Amp keeps MCP servers under this literal key in its settings file.
const FIELD: &str = "amp.mcpServers";

impl AppAdapter for AmpAdapter {
    fn app(&self) -> SupportedApp {
        SupportedApp::Amp
    }

    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)> {
        vec![
            (
                ctx.workspace_file(".amp/settings.json")
                    .to_string_lossy()
                    .to_string(),
                10,
            ),
            (
                ctx.user_app_config_path(self.app())
                    .to_string_lossy()
                    .to_string(),
                20,
            ),
        ]
    }

    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources {
        let normalized = match extract_json_field_mcp_json(content, FIELD) {
            Ok(value) => value,
            Err(error) => {
                return ParsedSources {
                    sources: vec![LocalConfigSource {
                        app: self.app().as_str().to_string(),
                        path: path.to_string(),
                        exists: true,
                        format: "json".to_string(),
                        priority,
                        content: Some(content.to_string()),
                    }],
                    servers: vec![],
                    warnings: vec![],
                    errors: vec![error],
                }
            }
        };
        let parsed = parse_mcp_json(&normalized);
        ParsedSources {
            sources: vec![LocalConfigSource {
                app: self.app().as_str().to_string(),
                path: path.to_string(),
                exists: true,
                format: "json".to_string(),
                priority,
                content: Some(normalized),
            }],
            servers: enable_servers_for_app_at(
                parsed.servers,
                self.app(),
                scope_for_path(ctx, path),
                path.to_string(),
            )
            .into_iter()
            .map(|server| (server, priority))
            .collect(),
            warnings: parsed.warnings,
            errors: parsed.errors,
        }
    }

    fn plan_apply(
        &self,
        ctx: &PlatformContext,
        config: &MCPConfig,
        previous_config: Option<&MCPConfig>,
    ) -> Vec<WriteOperation> {
        managed_json_field_writes(ctx, FIELD, self.app(), config, previous_config)
    }
}

#[cfg(test)]
mod tests {
    use super::AmpAdapter;
    use crate::adapters::AppAdapter;
    use crate::core::{empty_apps, MCPConfig, MCPServer, SupportedApp, TransportSpec};
    use crate::platform::{PlatformContext, PlatformOs};
    use crate::storage::apply_operation;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    const SETTINGS: &str = r#"{
  // Amp CLI settings
  "amp.notifications.enabled": true,
  "amp.mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["-y", "@playwright/mcp@latest", "--headless"],
    },
    "sourcegraph": {
      "url": "${SRC_ENDPOINT}/.api/mcp/v1",
      "headers": { "Authorization": "token ${SRC_ACCESS_TOKEN}" },
      "includeTools": ["keyword_search"],
    },
  },
}
"#;

    fn ctx(home: PathBuf) -> PlatformContext {
        PlatformContext {
            os: PlatformOs::Linux,
            home_dir: home.clone(),
            workspace_root: home,
        }
    }

    #[test]
    fn reads_user_and_workspace_settings() {
        let home = PathBuf::from("/home/test");
        let ctx = PlatformContext {
            workspace_root: PathBuf::from("/work/project"),
            ..ctx(home)
        };
        let sources = AmpAdapter.detect_sources(&ctx);
        // Compared by component: Windows joins with `\\`.
        let workspace = PathBuf::from(&sources[0].0);
        assert!(
            workspace.starts_with("/work/project") && workspace.ends_with(".amp/settings.json")
        );
        let user = PathBuf::from(&sources[1].0);
        assert!(user.starts_with("/home/test") && user.ends_with(".config/amp/settings.json"));

        let parsed = AmpAdapter.parse_source(&ctx, &sources[1].0, 20, SETTINGS);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        let sourcegraph = &parsed
            .servers
            .iter()
            .find(|(server, _)| server.id == "sourcegraph")
            .unwrap()
            .0;
        assert_eq!(
            sourcegraph.transport.headers["Authorization"],
            "token ${SRC_ACCESS_TOKEN}"
        );
    }

    #[test]
    fn apply_edits_the_settings_file_in_place() {
        let home = tempfile::tempdir().expect("tempdir");
        let ctx = ctx(home.path().to_path_buf());
        let path = ctx.user_app_config_path(SupportedApp::Amp);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, SETTINGS).unwrap();

        let parsed = AmpAdapter.parse_source(&ctx, &path.to_string_lossy(), 20, SETTINGS);
        let mut servers = parsed
            .servers
            .into_iter()
            .map(|(server, _)| server)
            .collect::<Vec<_>>();
        let mut apps = empty_apps();
        apps.insert(SupportedApp::Amp, true);
        servers.push(MCPServer {
            id: "linear".into(),
            name: "linear".into(),
            enabled: true,
            transport: TransportSpec {
                kind: "http".into(),
                url: Some("https://mcp.linear.app/mcp".into()),
                headers: Default::default(),
            },
            command: None,
            apps,
            placements: vec![],
            description: None,
            homepage: None,
        });
        let config = MCPConfig {
            version: 1,
            servers,
        };
        for operation in AmpAdapter.plan_apply(&ctx, &config, None) {
            apply_operation(&PathBuf::from(&operation.path), &operation).expect("apply");
        }

        let written = fs::read_to_string(&path).unwrap();
        assert!(
            written
                .starts_with("{\n  // Amp CLI settings\n  \"amp.notifications.enabled\": true,\n"),
            "{written}"
        );
        let settings = crate::parser::parse_jsonc(&written).expect("valid JSONC");
        let servers = &settings["amp.mcpServers"];
        assert_eq!(
            servers["linear"],
            json!({"url": "https://mcp.linear.app/mcp"})
        );
        // Amp's own tool filter stays.
        assert_eq!(
            servers["sourcegraph"]["includeTools"],
            json!(["keyword_search"])
        );
        assert!(servers["playwright"].get("type").is_none());
    }
}
