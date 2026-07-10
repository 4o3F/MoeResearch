//! Library surface for the `moeresearch` binary composition root.
//!
//! Kept `publish = false`. Exposed so `moe-research-tests` can cover pure
//! config→runtime mapping without embedding `#[cfg(test)]` modules in CLI
//! sources.

#![warn(clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod commands;
pub mod compose;
pub mod onboarding;
