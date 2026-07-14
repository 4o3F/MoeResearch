//! Network boundary for MoeResearch.

pub mod client;
pub(crate) mod log_safe;
pub mod provider_http;
pub mod reqwest_client;
pub mod types;

pub use client::NetworkClient;
pub use types::{
    DocumentNetworkOutcome, DocumentNetworkRejection, DocumentNetworkResponse, Header,
    JsonNetworkResponse, NetworkRequest, SseEvent, SseNetworkStream,
};
