#![warn(clippy::pedantic)]
#![allow(
    dead_code,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::struct_field_names
)]

pub mod config;
pub mod error;
pub mod logging;
pub mod mcp;
pub mod model;
pub mod net;
pub mod orchestrator;
pub mod schema;
pub mod search;
