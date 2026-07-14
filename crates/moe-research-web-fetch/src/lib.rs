//! Restricted public-document retrieval and focused prompt processing.

mod cache;
mod document;
mod flight;
mod model;
mod service;

pub use model::{WebFetchAnswer, WebFetchAnswerOutcome};
pub use service::{
    WebFetchDocument, WebFetchDocumentOutcome, WebFetchRuntimeConfig, WebFetchService,
    WebFetchSoftError,
};
