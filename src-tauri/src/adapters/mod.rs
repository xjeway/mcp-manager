mod claude_code;
mod codex;
mod cursor;
mod vscode;

use crate::core::{LocalConfigSource, MCPConfig, MCPServer, SupportedApp, WriteOperation};
use crate::platform::PlatformContext;

pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use cursor::CursorAdapter;
pub use vscode::VSCodeAdapter;

pub struct ParsedSources {
    pub sources: Vec<LocalConfigSource>,
    pub servers: Vec<(MCPServer, u32)>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub trait AppAdapter {
    fn app(&self) -> SupportedApp;
    fn detect_sources(&self, ctx: &PlatformContext) -> Vec<(String, u32)>;
    fn parse_source(
        &self,
        ctx: &PlatformContext,
        path: &str,
        priority: u32,
        content: &str,
    ) -> ParsedSources;
    fn plan_apply(&self, ctx: &PlatformContext, config: &MCPConfig) -> WriteOperation;
}

pub fn adapters() -> Vec<Box<dyn AppAdapter>> {
    vec![
        Box::new(VSCodeAdapter),
        Box::new(CursorAdapter),
        Box::new(ClaudeCodeAdapter),
        Box::new(CodexAdapter),
    ]
}
