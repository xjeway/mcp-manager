# mcpmgr

Add, list and remove MCP servers across Claude Code, Codex, Cursor, VS Code and more from the command line. `mcpmgr` shares its server list with the [MCP Manager](https://github.com/xjeway/mcp-manager) desktop app, so changes made in either show up in the other.

```bash
npx mcpmgr add context7 -- npx -y @upstash/context7-mcp@latest
npx mcpmgr add linear --url https://mcp.linear.app/mcp -a cursor,claude-code
npx mcpmgr list
npx mcpmgr remove context7 -a codex
npx mcpmgr rollback
```

Or install it globally with `npm install -g mcpmgr`, or with Homebrew: `brew install xjeway/mcp-manager/mcpmgr`.

In a terminal, `add` walks you through choosing servers, clients and user or project scope, then shows what each client file will get before writing. Every step can be answered with a flag; run `mcpmgr --help` for the full list.

This package is a small launcher. The native binary for your platform comes from one of the `@mcpmgr/*` optional dependencies.
