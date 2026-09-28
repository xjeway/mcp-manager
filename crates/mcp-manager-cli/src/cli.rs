use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    version,
    about = "Add, list and remove MCP servers across Claude Code, Codex, Cursor, VS Code and more",
    after_help = "Shares its server list with the MCP Manager desktop app.\n\nExamples:\n  add context7 -- npx -y @upstash/context7-mcp@latest\n  add linear --url https://mcp.linear.app/mcp -a cursor,claude-code\n  add --from mcp.json --project\n  remove context7 -a codex"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Answer every prompt with its default and skip the confirmation
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    /// Print a JSON result on stdout (with --yes or --dry-run for changes)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add MCP servers and install them into clients
    Add(AddArgs),
    /// List the servers and the clients they are installed in
    #[command(alias = "ls")]
    List,
    /// Remove servers, or take them out of some clients only
    #[command(alias = "rm")]
    Remove(RemoveArgs),
    /// Undo the last change made by this command-line tool
    Rollback,
}

#[derive(Args)]
pub struct AddArgs {
    /// Id for the new server (with --url or -- <command>)
    pub id: Option<String>,

    /// Remote server URL
    #[arg(long)]
    pub url: Option<String>,

    /// Remote transport
    #[arg(long, value_parser = ["http", "sse"], requires = "url")]
    pub transport: Option<String>,

    /// Read servers from a JSON config in any client's format ("-" for stdin)
    #[arg(long, value_name = "FILE")]
    pub from: Option<PathBuf>,

    /// Which servers from --from to add (default: ask, or all)
    #[arg(short = 's', long = "server", value_name = "ID")]
    pub servers: Vec<String>,

    /// Environment variable for stdio servers, repeatable
    #[arg(short = 'e', long = "env", value_name = "KEY=VALUE")]
    pub env: Vec<String>,

    /// Clients to install into, e.g. cursor,claude-code, or '*' for all
    #[arg(
        short = 'a',
        long = "app",
        value_name = "CLIENT",
        value_delimiter = ','
    )]
    pub apps: Vec<String>,

    /// Use each client's user-level config
    #[arg(short = 'g', long, conflicts_with = "project")]
    pub global: bool,

    /// Use the current project's config files
    #[arg(short = 'p', long)]
    pub project: bool,

    /// Show what would change without writing anything
    #[arg(long)]
    pub dry_run: bool,

    /// Command that starts a stdio server
    #[arg(last = true, value_name = "COMMAND")]
    pub command: Vec<String>,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// Servers to remove (default: ask)
    pub ids: Vec<String>,

    /// Only take the servers out of these clients
    #[arg(
        short = 'a',
        long = "app",
        value_name = "CLIENT",
        value_delimiter = ','
    )]
    pub apps: Vec<String>,

    /// Show what would change without writing anything
    #[arg(long)]
    pub dry_run: bool,
}
