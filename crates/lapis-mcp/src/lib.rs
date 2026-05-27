//! MCP adapter boundary for Lapis.

pub mod envelope;
pub mod server;
pub mod tools;

pub use envelope::{ToolEnvelope, ToolError, ToolErrorCode, ToolStatus};
pub use server::{LapisMcpServer, serve_stdio};
