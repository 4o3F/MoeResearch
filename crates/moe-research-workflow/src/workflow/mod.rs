//! Workflow orchestration for standalone aspect and multi-aspect deep research.
//!
//! This module owns the execution boundary: validate incoming requests, derive
//! the effective research limits from operator config and request limits, run
//! aspect agents, then aggregate successes and failures into the public result.

mod aggregation;
mod aspect;
mod deep;

pub use aspect::{AspectResearchFailure, AspectResearchOutput, aspect_research};
pub use deep::{DeepResearchFailure, deep_research};
