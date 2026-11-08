//! Desktop project boundaries. CLI-wide operations retain their existing semantics.

use crate::core::{MCPConfig, PlacementScope, ServerPlacement, SupportedApp};
use crate::platform::PlatformContext;
use crate::storage::target_identity;
use std::collections::BTreeMap;
use std::path::Component;

/// Whether a placement belongs to the selected project or the computer-wide scope.
pub fn placement_in_context(ctx: &PlatformContext, placement: &ServerPlacement) -> bool {
    if placement.scope == PlacementScope::User {
        return true;
    }
    if !ctx.has_workspace() {
        return false;
    }
    // A legacy pathless placement has no owner. Import its concrete client file
    // before allowing desktop writes; never rebind it when switching projects.
    let Some(path) = placement.path.as_ref() else {
        return false;
    };
    let path = ctx.resolve_path(path);
    if path.components().any(|part| part == Component::ParentDir) {
        return false;
    }
    let root = target_identity(&ctx.workspace_root);
    let identity = target_identity(&path);
    if identity.starts_with(&root)
        && project_paths(ctx, placement.app)
            .iter()
            .any(|candidate| target_identity(candidate) == identity)
    {
        return true;
    }
    // Imported parent Agents definitions remain visible but read-only.
    !placement.managed
        && placement.app == SupportedApp::Agents
        && root
            .parent()
            .is_some_and(|parent| identity == target_identity(&parent.join(".agents/mcp.json")))
}

/// Concrete client paths owned by this project, excluding nested project files.
pub fn project_paths(ctx: &PlatformContext, app: SupportedApp) -> Vec<std::path::PathBuf> {
    if !ctx.has_workspace() {
        return Vec::new();
    }
    let mut paths = crate::adapters::adapters()
        .into_iter()
        .find(|adapter| adapter.app() == app)
        .map(|adapter| {
            adapter
                .detect_sources(ctx)
                .into_iter()
                .map(|(path, _)| ctx.resolve_path(&path))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(path) = crate::adapters::workspace_scope_path(ctx, app) {
        paths.push(ctx.resolve_path(&path));
    }
    let root = target_identity(&ctx.workspace_root);
    paths.retain(|path| {
        path.starts_with(&ctx.workspace_root) && target_identity(path).starts_with(&root)
    });
    paths.sort();
    paths.dedup();
    paths
}

/// Filter only the native write inputs, leaving the shared store and other projects intact.
pub fn scoped_config(ctx: &PlatformContext, config: &MCPConfig) -> MCPConfig {
    let mut scoped = config.clone();
    scoped.servers.retain_mut(|server| {
        let before = server.placements.len();
        server.placements.retain(|p| placement_in_context(ctx, p));
        !(before != server.placements.len()
            && server.placements.is_empty()
            && !server.apps.values().any(|enabled| *enabled))
    });
    scoped
}

/// Refuse removal, disabling or reassignment of another project's saved placements.
pub fn validate_foreign_placements(
    ctx: &PlatformContext,
    previous: &MCPConfig,
    next: &MCPConfig,
) -> Result<(), String> {
    let foreign = |config: &MCPConfig| {
        config
            .servers
            .iter()
            .filter_map(|server| {
                let mut placements: Vec<_> = server
                    .placements
                    .iter()
                    .filter(|p| !placement_in_context(ctx, p))
                    .map(|p| serde_json::to_string(p).expect("placement serialization"))
                    .collect();
                placements.sort();
                (!placements.is_empty()).then(|| (server.id.clone(), (server.enabled, placements)))
            })
            .collect::<BTreeMap<_, _>>()
    };
    if foreign(previous) != foreign(next) {
        return Err("This change affects another project's configuration. Select that project before changing its placements.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{AppAdapter, CursorAdapter};
    use crate::core::{PlacementScope, ServerPlacement};
    use crate::platform::PlatformOs;
    use std::fs;

    fn setup() -> (tempfile::TempDir, PlatformContext, ServerPlacement) {
        let temp = tempfile::tempdir().unwrap();
        let ctx = PlatformContext {
            os: PlatformOs::MacOS,
            home_dir: temp.path().join("home"),
            workspace_root: temp.path().join("a"),
        };
        fs::create_dir_all(&ctx.workspace_root).unwrap();
        let foreign = ServerPlacement {
            app: SupportedApp::Cursor,
            scope: PlacementScope::Workspace,
            path: Some(
                temp.path()
                    .join("b/.cursor/mcp.json")
                    .to_string_lossy()
                    .into(),
            ),
            enabled: true,
            managed: true,
        };
        (temp, ctx, foreign)
    }

    fn config(placements: Vec<ServerPlacement>) -> MCPConfig {
        let mut servers =
            crate::parser::parse_mcp_json(r#"{"mcpServers":{"shared":{"command":"node"}}}"#)
                .servers;
        servers[0].placements = placements;
        MCPConfig {
            version: 1,
            servers,
        }
    }

    #[test]
    fn plans_only_selected_project_and_preserves_foreign_store_metadata() {
        let (_temp, ctx, foreign) = setup();
        let local = ServerPlacement {
            path: Some(
                ctx.workspace_file(".cursor/mcp.json")
                    .to_string_lossy()
                    .into(),
            ),
            ..foreign.clone()
        };
        let all = config(vec![foreign.clone(), local]);
        assert_eq!(
            CursorAdapter.plan_apply(&ctx, &all, None).len(),
            2,
            "CLI-wide planning remains supported"
        );
        let scoped = scoped_config(&ctx, &all);
        let operations = CursorAdapter.plan_apply(&ctx, &scoped, None);
        assert_eq!(operations.len(), 1);
        assert_eq!(
            operations[0].path,
            ctx.workspace_file(".cursor/mcp.json").to_string_lossy()
        );
        assert_eq!(all.servers[0].placements.len(), 2);
        let mut none = ctx.clone();
        none.workspace_root = Default::default();
        assert!(scoped_config(&none, &all).servers.is_empty());
    }

    #[test]
    fn nested_and_pathless_placements_cannot_follow_the_selection() {
        let (_temp, ctx, foreign) = setup();
        let mut nested = foreign.clone();
        nested.path = Some(
            ctx.workspace_file("nested/.cursor/mcp.json")
                .to_string_lossy()
                .into(),
        );
        assert!(!placement_in_context(&ctx, &nested));
        let mut child = ctx.clone();
        child.workspace_root = ctx.workspace_file("nested");
        fs::create_dir_all(&child.workspace_root).unwrap();
        assert!(placement_in_context(&child, &nested));
        let mut ambiguous = nested.clone();
        ambiguous.path = None;
        assert!(!placement_in_context(&ctx, &ambiguous));
        assert!(!placement_in_context(&child, &ambiguous));
        let previous = config(vec![nested, ambiguous]);
        assert_eq!(scoped_config(&ctx, &previous).servers.len(), 0);
        assert!(validate_foreign_placements(
            &ctx,
            &previous,
            &MCPConfig {
                version: 1,
                servers: vec![]
            }
        )
        .is_err());
        assert!(CursorAdapter
            .plan_apply(&ctx, &scoped_config(&ctx, &previous), None)
            .is_empty());
    }

    #[test]
    fn refuses_foreign_removal_reassignment_and_shared_disabling() {
        let (_temp, ctx, foreign) = setup();
        let previous = config(vec![foreign.clone()]);
        assert!(validate_foreign_placements(
            &ctx,
            &previous,
            &MCPConfig {
                version: 1,
                servers: vec![]
            }
        )
        .is_err());
        let mut next = previous.clone();
        next.servers[0].enabled = false;
        assert!(validate_foreign_placements(&ctx, &previous, &next).is_err());
        next = previous.clone();
        next.servers[0].placements[0].enabled = false;
        assert!(validate_foreign_placements(&ctx, &previous, &next).is_err());
        next = previous.clone();
        next.servers[0].command.as_mut().unwrap().program = "new-shared-definition".into();
        validate_foreign_placements(&ctx, &previous, &next).unwrap();
        let empty = MCPConfig {
            version: 1,
            servers: vec![],
        };
        assert!(validate_foreign_placements(&ctx, &empty, &previous).is_err());
    }

    #[test]
    fn recorded_project_apply_and_undo_never_touch_the_other_project() {
        let (temp, ctx, foreign) = setup();
        let other = ctx.resolve_path(foreign.path.as_ref().unwrap());
        fs::create_dir_all(other.parent().unwrap()).unwrap();
        fs::write(&other, "external project b").unwrap();
        let previous = config(vec![foreign.clone()]);
        let local = ServerPlacement {
            path: Some(
                ctx.workspace_file(".cursor/mcp.json")
                    .to_string_lossy()
                    .into(),
            ),
            ..foreign
        };
        let next = config(vec![previous.servers[0].placements[0].clone(), local]);
        let store = crate::store::config_path(&ctx);
        let fingerprint =
            crate::store::write_text_checked(&store, &serde_yaml::to_string(&next).unwrap(), None)
                .unwrap();
        crate::history::apply_recorded_in_project(&ctx, &next, &previous, Some(&fingerprint), None)
            .unwrap();
        assert!(ctx.workspace_file(".cursor/mcp.json").exists());
        assert_eq!(fs::read_to_string(&other).unwrap(), "external project b");
        crate::history::rollback_last(&ctx).unwrap();
        assert!(!ctx.workspace_file(".cursor/mcp.json").exists());
        assert_eq!(fs::read_to_string(&other).unwrap(), "external project b");
        assert_eq!(
            crate::store::load(&ctx).unwrap().servers[0]
                .placements
                .len(),
            1
        );
        drop(temp);
    }

    #[cfg(unix)]
    #[test]
    fn project_symlinks_cannot_expand_the_writable_context() {
        let (temp, ctx, mut placement) = setup();
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, ctx.workspace_file("link")).unwrap();
        placement.path = Some(ctx.workspace_file("link/mcp.json").to_string_lossy().into());
        assert!(!placement_in_context(&ctx, &placement));
        placement.path = Some(ctx.workspace_file("../b/mcp.json").to_string_lossy().into());
        assert!(!placement_in_context(&ctx, &placement));
    }
}
