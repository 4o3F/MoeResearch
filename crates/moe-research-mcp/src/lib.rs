//! MCP adapter boundary for MoeResearch.

pub mod envelope;
pub mod server;
pub mod tools;

pub use envelope::{ToolEnvelope, ToolError, ToolErrorCode, ToolStatus};
pub use server::{MoeResearchMcpServer, serve_stdio};
