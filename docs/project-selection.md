# Selecting a project in the desktop app

Use the folder selector in the dashboard to choose a project directory. The selector remembers the last selection and up to five recent projects on this computer. Switching projects does not import or write configuration; use **Import** to discover that project's existing servers.

The dashboard shows computer-wide installations together with installations belonging to the selected project. **My computer** shows computer-wide installations without selecting a project. Computer-wide installations are shared across projects.

Saving, disabling, or deleting a server applies client-file changes only to computer-wide installations and the selected project's installations. Saved placements and client files for other projects are preserved. The app rechecks that the selected directory is available before applying and before each client-file write. These checks are optimistic: external processes do not participate in the transaction and may race after a check. Installations in nested project folders are preserved when selecting their parent. Legacy workspace placements without a concrete file path are retained but excluded from desktop writes; import their actual client files to resolve ownership. Inherited parent `.agents/mcp.json` entries remain read-only.

Server definitions are still shared in the unified configuration: changing a server's command, arguments, environment, or URL changes its canonical definition. Other projects' client files are updated only when you select and apply those projects. Independent definitions per project are outside this feature's scope. A server disabled globally cannot be re-enabled from a single project while it retains other project placements.

**Undo last change (all projects)** uses the existing shared history. It restores the most recent recorded change, which may belong to a different project. Project selection does not create a separate undo history.

CLI commands keep their existing workspace and apply behavior. This selector affects desktop operations only.
