//! Research request domain, effective plans, and prompt projection.

mod plan;
mod prompt;
mod request;

pub use request::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask,
};

pub(crate) use plan::{
    EffectiveAspectPlan, EffectiveResearchPlan, WorkflowValidationContext,
    effective_research_limits,
};
pub(crate) use prompt::{ASPECT_PROMPT_MAX_BYTES, AspectPromptInput};
pub(crate) use request::SUPPORTED_SCHEMA_VERSIONS;
