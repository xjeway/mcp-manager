# @mcpmgr/cli

Add, list and remove MCP servers across Claude Code, Codex, Cursor, VS Code and more from the command line. `mcpmgr` shares its server list with the [MCP Manager](https://github.com/xjeway/mcp-manager) desktop app, so changes made in either show up in the other.

```bash
npx @mcpmgr/cli add context7 -- npx -y @upstash/context7-mcp@latest
npx @mcpmgr/cli add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
npx @mcpmgr/cli list
npx @mcpmgr/cli remove context7 -a codex
npx @mcpmgr/cli rollback
```

Or install the `mcpmgr` command globally with `npm install -g @mcpmgr/cli`, or with Homebrew: `brew install xjeway/mcp-manager/mcpmgr`.

In a terminal, `add` walks you through choosing servers, clients and user or project scope, then shows what each client file will get before writing. Every step can be answered with a flag; run `mcpmgr --help` for the full list.

This package is a small launcher. The native binary for your platform comes from one of the `@mcpmgr/*` optional dependencies.
