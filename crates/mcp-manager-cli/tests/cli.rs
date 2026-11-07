//! Runs the built binary against a throwaway home directory and project.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct Sandbox {
    _dir: TempDir,
    home: PathBuf,
    project: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let home = dir.path().join("home");
        let project = dir.path().join("project");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(&project).unwrap();
        Sandbox {
            _dir: dir,
            home,
            project,
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mcpmgr"));
        command
            .args(args)
            .current_dir(&self.project)
            .env_clear()
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("PATH", self.home.join("bin"))
            .env("MCP_MANAGER_SKIP_GLOBAL_COMMAND_DIRS", "1")
            // No terminal: prompts are impossible, as in CI or an agent.
            .stdin(Stdio::null());
        if let Some(root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", root);
        }
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("run cli")
    }

    fn ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }

    /// What the command printed, prompts and messages included.
    fn all_output(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(output.status.success(), "{args:?} failed");
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }

    fn json(&self, args: &[&str]) -> Value {
        serde_json::from_str(&self.ok(args)).expect("json output")
    }

    fn read_json(&self, path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).expect("read client config")).unwrap()
    }

    fn cursor_servers(&self) -> Value {
        self.read_json(&self.home.join(".cursor/mcp.json"))["mcpServers"].clone()
    }

    fn codex_config(&self) -> String {
        fs::read_to_string(self.home.join(".codex/config.toml")).unwrap_or_default()
    }

    /// The app's shared server list, where PlatformContext puts it on each OS.
    fn servers_yaml(&self) -> PathBuf {
        let data = if cfg!(target_os = "macos") {
            "Library/Application Support/mcp-manager"
        } else if cfg!(windows) {
            "AppData/Roaming/mcp-manager"
        } else {
            ".config/mcp-manager"
        };
        self.home.join(data).join("config/servers.yaml")
    }

    fn listed_ids(&self) -> Vec<String> {
        self.json(&["list", "--json"])
            .as_array()
            .unwrap()
            .iter()
            .map(|server| server["id"].as_str().unwrap().to_string())
            .collect()
    }
}

const ADD_CTX: &[&str] = &[
    "add",
    "ctx",
    "-a",
    "cursor,codex",
    "-y",
    "--",
    "npx",
    "-y",
    "ctx-mcp",
];

#[test]
fn add_writes_every_chosen_client_and_the_shared_list() {
    let sandbox = Sandbox::new();

    sandbox.ok(ADD_CTX);

    let entry = &sandbox.cursor_servers()["ctx"];
    assert_eq!(entry["command"], "npx");
    assert_eq!(entry["args"], serde_json::json!(["-y", "ctx-mcp"]));
    assert!(sandbox.codex_config().contains("[mcp_servers.ctx]"));

    let listed = sandbox.json(&["list", "--json"]);
    let clients = listed[0]["clients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["client"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(clients, vec!["cursor", "codex"]);
}

#[test]
fn dry_run_reports_without_writing() {
    let sandbox = Sandbox::new();

    let report = sandbox.json(&[
        "add",
        "ctx",
        "-a",
        "cursor",
        "--dry-run",
        "--json",
        "--",
        "npx",
        "ctx-mcp",
    ]);

    assert_eq!(report["applied"], false);
    assert_eq!(report["changes"][0]["serverId"], "ctx");
    assert_eq!(report["changes"][0]["action"], "add");
    assert!(!sandbox.home.join(".cursor/mcp.json").exists());
    assert!(sandbox.listed_ids().is_empty());
}

#[test]
fn json_changes_need_yes_or_dry_run() {
    let sandbox = Sandbox::new();

    let output = sandbox.run(&["add", "ctx", "-a", "cursor", "--json", "--", "npx"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--yes"));
    assert!(!sandbox.home.join(".cursor/mcp.json").exists());
}

#[test]
fn flags_the_overwrite_of_an_entry_added_by_hand() {
    let sandbox = Sandbox::new();
    fs::create_dir_all(sandbox.home.join(".cursor")).unwrap();
    fs::write(
        sandbox.home.join(".cursor/mcp.json"),
        r#"{ "mcpServers": { "ctx": { "command": "mine" }, "other": { "command": "keep" } } }"#,
    )
    .unwrap();

    let report = sandbox.json(&[
        "add",
        "ctx",
        "-a",
        "cursor",
        "--dry-run",
        "--json",
        "--",
        "npx",
        "ctx-mcp",
    ]);
    assert_eq!(report["changes"][0]["action"], "overwriteUnmanaged");

    sandbox.ok(&["add", "ctx", "-a", "cursor", "-y", "--", "npx", "ctx-mcp"]);
    let servers = sandbox.cursor_servers();
    assert_eq!(servers["ctx"]["command"], "npx");
    assert_eq!(
        servers["other"]["command"], "keep",
        "unrelated entries survive"
    );
}

#[test]
fn remove_from_one_client_then_roll_back_then_remove_everywhere() {
    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);

    sandbox.ok(&["remove", "ctx", "-a", "codex", "-y"]);
    assert!(!sandbox.codex_config().contains("ctx"));
    assert!(sandbox.cursor_servers().get("ctx").is_some());

    sandbox.ok(&["rollback", "-y"]);
    assert!(sandbox.codex_config().contains("[mcp_servers.ctx]"));
    let clients = sandbox.json(&["list", "--json"])[0]["clients"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(clients, 2, "the shared list is restored too");

    sandbox.ok(&["remove", "ctx", "-y"]);
    assert!(sandbox.cursor_servers().get("ctx").is_none());
    assert!(!sandbox.codex_config().contains("ctx"));
    assert!(sandbox.listed_ids().is_empty());
}

#[test]
fn project_scope_writes_the_project_file_only() {
    let sandbox = Sandbox::new();

    sandbox.ok(&[
        "add",
        "linear",
        "--url",
        "https://mcp.linear.app/mcp",
        "-a",
        "claude-code,codex,devin,kimi-cli",
        "--project",
        "-y",
    ]);

    let project = sandbox.read_json(&sandbox.project.join(".mcp.json"));
    assert_eq!(
        project["mcpServers"]["linear"]["url"],
        "https://mcp.linear.app/mcp"
    );
    let user = fs::read_to_string(sandbox.home.join(".claude.json")).unwrap_or_default();
    assert!(!user.contains("linear"));
    // Codex loads `.codex/config.toml` for trusted projects.
    let codex_project = fs::read_to_string(sandbox.project.join(".codex/config.toml")).unwrap();
    assert!(codex_project.contains("[mcp_servers.linear]"));
    assert!(!sandbox.codex_config().contains("linear"));
    let devin = sandbox.read_json(&sandbox.project.join(".devin/mcp_config.json"));
    assert_eq!(
        devin["mcpServers"]["linear"]["serverUrl"],
        "https://mcp.linear.app/mcp"
    );
    assert!(!sandbox
        .home
        .join(".codeium/windsurf/mcp_config.json")
        .exists());
    // Kimi CLI has no project-level config, so it falls back to user level.
    let kimi = fs::read_to_string(sandbox.home.join(".kimi/mcp.json")).unwrap();
    assert!(kimi.contains("linear"));

    let clients = sandbox.json(&["list", "--json"])[0]["clients"].clone();
    assert_eq!(clients[0]["scope"], "project");
    assert_eq!(clients[0]["path"], "./.mcp.json");
}

#[test]
fn imports_selected_servers_from_a_json_file() {
    let sandbox = Sandbox::new();
    let file = sandbox.project.join("shared.json");
    fs::write(
        &file,
        r#"{ "mcpServers": {
            "github": { "command": "npx", "args": ["gh-mcp"], "env": { "GITHUB_TOKEN": "" } },
            "linear": { "url": "https://mcp.linear.app/mcp" }
        } }"#,
    )
    .unwrap();

    sandbox.ok(&[
        "add",
        "--from",
        "shared.json",
        "-s",
        "github",
        "-e",
        "GITHUB_TOKEN=abc",
        "-a",
        "cursor",
        "-y",
    ]);

    assert_eq!(sandbox.listed_ids(), vec!["github"]);
    assert_eq!(
        sandbox.cursor_servers()["github"]["env"]["GITHUB_TOKEN"],
        "abc"
    );
}

#[test]
fn inside_an_agent_it_targets_that_agents_client_without_prompting() {
    let sandbox = Sandbox::new();

    let output = sandbox
        .command(&["add", "ctx", "--", "npx", "ctx-mcp"])
        .env("CLAUDECODE", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let user = fs::read_to_string(sandbox.home.join(".claude.json")).unwrap();
    assert!(user.contains("ctx-mcp"));
    assert!(!sandbox.home.join(".cursor/mcp.json").exists());
}

#[test]
fn a_missing_answer_fails_with_the_flag_to_pass() {
    let sandbox = Sandbox::new();
    // Clients cannot be chosen: none is detected and no prompt is possible.
    let output = sandbox.run(&["add", "ctx", "--", "npx"]);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--app"), "{stderr}");
    assert!(sandbox.listed_ids().is_empty());
}

#[test]
fn rejects_unknown_clients_and_servers() {
    let sandbox = Sandbox::new();

    let output = sandbox.run(&["add", "ctx", "-a", "emacs", "-y", "--", "npx"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("claude-code"));

    let output = sandbox.run(&["remove", "nope", "-y"]);
    assert!(!output.status.success());
}

#[test]
fn rollback_refuses_to_undo_over_a_later_edit() {
    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);

    // The desktop app edits the list after the CLI's change.
    let yaml = fs::read_to_string(sandbox.servers_yaml()).unwrap();
    fs::write(
        sandbox.servers_yaml(),
        yaml.replace("ctx-mcp", "edited-in-app"),
    )
    .unwrap();

    let output = sandbox.run(&["rollback", "-y"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("changed after the last change"));
    let yaml = fs::read_to_string(sandbox.servers_yaml()).unwrap();
    assert!(yaml.contains("edited-in-app"), "the later edit is kept");
    assert!(
        sandbox.cursor_servers().get("ctx").is_some(),
        "client files are untouched"
    );
}

#[cfg(unix)]
#[test]
fn client_files_are_restored_when_the_server_list_cannot_be_saved() {
    use std::os::unix::fs::PermissionsExt;

    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);
    let cursor_before = fs::read_to_string(sandbox.home.join(".cursor/mcp.json")).unwrap();

    // servers.yaml is saved by writing a temporary file next to it, which a
    // read-only directory refuses, after the client files are written.
    let config_dir = sandbox.servers_yaml().parent().unwrap().to_path_buf();
    fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o555)).unwrap();
    // Privileged users (e.g. root in a container) can still write there, so
    // the save would not fail and there is nothing to test.
    let probe = config_dir.join("probe");
    if fs::write(&probe, "").is_ok() {
        let _ = fs::remove_file(&probe);
        fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o755)).unwrap();
        eprintln!("skipped: read-only directories are writable for this user");
        return;
    }
    let output = sandbox.run(&[
        "add",
        "other",
        "-a",
        "cursor",
        "-y",
        "--",
        "npx",
        "other-mcp",
    ]);
    fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o755)).unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Client files were restored"), "{stderr}");
    assert_eq!(
        fs::read_to_string(sandbox.home.join(".cursor/mcp.json")).unwrap(),
        cursor_before
    );
    assert_eq!(sandbox.listed_ids(), vec!["ctx"]);
}

#[test]
fn rollback_undoes_several_changes_one_after_another() {
    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);
    sandbox.ok(&["add", "two", "-a", "cursor", "-y", "--", "npx", "two-mcp"]);
    assert!(sandbox.cursor_servers().get("two").is_some());

    let output = sandbox.all_output(&["rollback", "-y"]);
    assert!(
        output.contains("1 earlier change can still be undone"),
        "{output}"
    );
    assert!(sandbox.cursor_servers().get("two").is_none());
    assert!(sandbox.cursor_servers().get("ctx").is_some());
    assert_eq!(sandbox.listed_ids(), vec!["ctx"]);

    sandbox.ok(&["rollback", "-y"]);
    assert!(sandbox.listed_ids().is_empty());
    assert!(sandbox
        .all_output(&["rollback", "-y"])
        .contains("Nothing to roll back"));
}

#[test]
fn rollback_removes_the_files_and_folders_the_change_created() {
    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);
    assert!(sandbox.home.join(".cursor/mcp.json").exists());
    assert!(sandbox.home.join(".codex/config.toml").exists());

    let output = sandbox.all_output(&["rollback", "-y"]);

    assert!(output.contains("Removed 2 client files"), "{output}");
    assert!(!sandbox.home.join(".cursor").exists());
    assert!(!sandbox.home.join(".codex").exists());
}

#[test]
fn rollback_can_finish_after_it_was_interrupted() {
    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);

    // As if a rollback had put one client file back and then been killed.
    fs::remove_file(sandbox.home.join(".codex/config.toml")).unwrap();

    sandbox.ok(&["rollback", "-y"]);

    assert!(!sandbox.home.join(".cursor/mcp.json").exists());
    assert!(sandbox.listed_ids().is_empty());
}

#[test]
fn rollback_refuses_to_discard_a_later_edit_of_a_client_file() {
    let sandbox = Sandbox::new();
    sandbox.ok(ADD_CTX);

    // Someone edits a client file by hand; servers.yaml stays as the CLI left it.
    let cursor = sandbox.home.join(".cursor/mcp.json");
    let mut edited = sandbox.read_json(&cursor);
    edited["mcpServers"]["mine"] = serde_json::json!({ "command": "mine" });
    fs::write(&cursor, edited.to_string()).unwrap();

    let output = sandbox.run(&["rollback", "-y"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("were edited") && stderr.contains("mcp.json"),
        "{stderr}"
    );
    assert_eq!(sandbox.cursor_servers()["mine"]["command"], "mine");
    assert!(
        sandbox.codex_config().contains("[mcp_servers.ctx]"),
        "nothing was rolled back"
    );
}

#[test]
fn keeps_what_a_newer_version_saved_and_will_not_write_a_newer_format() {
    let sandbox = Sandbox::new();
    let yaml = sandbox.servers_yaml();
    fs::create_dir_all(yaml.parent().unwrap()).unwrap();
    fs::write(
        &yaml,
        "version: 1\nprofiles: [work]\nservers:\n- id: linear\n  name: linear\n  enabled: true\n  \
         tags: [work]\n  transport:\n    type: http\n    url: https://mcp.linear.app/mcp\n  \
         apps:\n    cursor: false\n    zed: true\n",
    )
    .unwrap();

    assert_eq!(sandbox.listed_ids(), ["linear"]);
    sandbox.ok(ADD_CTX);
    let saved = fs::read_to_string(&yaml).unwrap();
    for kept in ["profiles:", "tags:", "zed: true"] {
        assert!(saved.contains(kept), "{kept} was dropped:\n{saved}");
    }

    let newer = saved.replace("version: 1", "version: 2");
    fs::write(&yaml, &newer).unwrap();
    let output = sandbox.run(&["remove", "ctx", "-y"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("newer version"), "{stderr}");
    assert!(!stderr.contains("CONFIG_TOO_NEW"), "{stderr}");
    assert_eq!(fs::read_to_string(&yaml).unwrap(), newer);
    assert!(sandbox.cursor_servers().get("ctx").is_some());
    assert!(sandbox.listed_ids().contains(&"ctx".to_string()));
}

#[test]
fn reports_a_server_a_client_cannot_take() {
    let sandbox = Sandbox::new();

    let report = sandbox.json(&[
        "add",
        "events",
        "-a",
        "goose",
        "--url",
        "https://example.com/sse",
        "--transport",
        "sse",
        "-y",
        "--json",
    ]);

    // Goose's file has nothing to write, but the server is still reported.
    assert_eq!(report["applied"], false);
    assert!(!sandbox.home.join(".config/goose/config.yaml").exists());
    assert_eq!(report["warnings"][0]["kind"], "sseUnsupported");
    assert_eq!(report["warnings"][0]["app"], "goose");
    assert_eq!(report["warnings"][0]["serverId"], "events");
}

#[test]
fn project_secret_warning_is_nonblocking_and_redacted() {
    let sandbox = Sandbox::new();
    let report = sandbox.json(&[
        "add",
        "github",
        "-a",
        "claude-code",
        "--project",
        "--env",
        "GITHUB_TOKEN=private-test-token",
        "-y",
        "--json",
        "--",
        "node",
        "server.js",
    ]);
    assert_eq!(report["warnings"][0]["kind"], "sharedProjectSecret");
    assert_eq!(report["warnings"][0]["key"], "GITHUB_TOKEN");
    assert!(report["warnings"][0]["path"]
        .as_str()
        .unwrap()
        .ends_with(".mcp.json"));
    assert!(!report["warnings"]
        .to_string()
        .contains("private-test-token"));
    let written = sandbox.read_json(&sandbox.project.join(".mcp.json"));
    assert_eq!(
        written["mcpServers"]["github"]["env"]["GITHUB_TOKEN"],
        "private-test-token"
    );

    let reference = Sandbox::new();
    let report = reference.json(&[
        "add",
        "github",
        "-a",
        "claude-code",
        "--project",
        "--env",
        "GITHUB_TOKEN=${TOKEN}",
        "-y",
        "--json",
        "--",
        "node",
        "server.js",
    ]);
    assert_eq!(report["warnings"], serde_json::json!([]));

    let user = Sandbox::new();
    let report = user.json(&[
        "add",
        "github",
        "-a",
        "claude-code",
        "--global",
        "--env",
        "GITHUB_TOKEN=private-test-token",
        "-y",
        "--json",
        "--",
        "node",
        "server.js",
    ]);
    assert_eq!(report["warnings"], serde_json::json!([]));
}

#[test]
fn project_secret_warning_is_human_readable_and_redacted() {
    let sandbox = Sandbox::new();
    let output = sandbox.all_output(&[
        "add",
        "github",
        "-a",
        "claude-code",
        "--project",
        "--env",
        "GITHUB_TOKEN=private-test-token",
        "-y",
        "--",
        "node",
        "server.js",
    ]);
    assert!(output.contains("GITHUB_TOKEN"));
    assert!(output.contains("Committing this file could expose it"));
    assert!(output.contains("${VAR}"));
    assert!(!output.contains("private-test-token"));
}

#[test]
fn auggie_cli_writes_user_and_project_settings_in_place() {
    let sandbox = Sandbox::new();
    let user_path = sandbox.home.join(".augment/settings.json");
    fs::create_dir_all(user_path.parent().unwrap()).unwrap();
    fs::write(&user_path, "{\n// preserve this\n\"theme\":\"dark\",\"mcpServers\":{\"existing\":{\"command\":\"old\"}}\n}").unwrap();
    sandbox.ok(&[
        "add",
        "context",
        "-a",
        "auggie",
        "--global",
        "-y",
        "--",
        "node",
        "server.js",
    ]);
    let text = fs::read_to_string(&user_path).unwrap();
    assert!(text.contains("// preserve this"));
    assert!(text.contains("\"theme\":\"dark\""));
    assert!(text.contains("\"existing\":{\"command\":\"old\"}"));
    assert!(text.contains("server.js"));
    assert!(!sandbox.project.join(".augment/settings.json").exists());

    let report = sandbox.json(&[
        "add",
        "remote",
        "--url",
        "https://example.com/mcp",
        "-a",
        "auggie",
        "--project",
        "-y",
        "--json",
    ]);
    assert_eq!(report["warnings"], serde_json::json!([]));
    let project = sandbox.read_json(&sandbox.project.join(".augment/settings.json"));
    assert_eq!(project["mcpServers"]["remote"]["type"], "http");
    assert_eq!(
        project["mcpServers"]["remote"]["url"],
        "https://example.com/mcp"
    );
    assert!(project["mcpServers"]["remote"].get("transport").is_none());
}

#[test]
fn codebuddy_add_updates_only_the_active_user_file() {
    for selected in 0..3 {
        let sandbox = Sandbox::new();
        let paths = [
            ".codebuddy/.mcp.json",
            ".codebuddy/mcp.json",
            ".codebuddy.json",
        ]
        .map(|relative| sandbox.home.join(relative));
        for (index, path) in paths.iter().enumerate().skip(selected) {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, format!("{{\n// keep active file\n\"marker\":{index},\"projects\":{{\"private\":{{\"mcpServers\":{{\"local\":{{\"command\":\"private\"}}}}}}}},\"mcpServers\":{{\"old{index}\":{{\"command\":\"old\"}}}}\n}}")).unwrap();
        }
        let before: Vec<_> = paths.iter().map(|path| fs::read(path).ok()).collect();
        sandbox.ok(&[
            "add",
            "new",
            "-a",
            "codebuddy",
            "--global",
            "-y",
            "--",
            "node",
            "server.js",
        ]);
        for (index, path) in paths.iter().enumerate() {
            if index == selected {
                let written = fs::read_to_string(path).unwrap();
                assert!(written.contains("// keep active file"));
                assert!(written.contains("server.js"));
                assert!(written.contains(&format!("old{selected}")));
                assert!(written.contains("\"command\":\"private\""));
            } else {
                assert_eq!(fs::read(path).ok(), before[index]);
            }
        }
    }
}

#[test]
fn codebuddy_new_install_defaults_to_preferred_path_and_leaves_shared_project_file_to_claude() {
    let sandbox = Sandbox::new();
    let shared_path = sandbox.project.join(".mcp.json");
    let shared = r#"{"mcpServers":{"shared":{"command":"keep"}}}"#;
    fs::write(&shared_path, shared).unwrap();
    sandbox.ok(&[
        "add",
        "api",
        "--url",
        "https://example.com/mcp",
        "-a",
        "codebuddy",
        "--global",
        "-y",
    ]);
    let user = sandbox.read_json(&sandbox.home.join(".codebuddy/.mcp.json"));
    assert_eq!(user["mcpServers"]["api"]["type"], "http");
    assert_eq!(fs::read_to_string(&shared_path).unwrap(), shared);
    assert!(!sandbox.home.join(".codebuddy/mcp.json").exists());
    assert!(!sandbox.home.join(".codebuddy.json").exists());
}
