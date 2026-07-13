//! Research request domain, effective plans, and prompt projection.

mod plan;
mod prompt;
mod request;

pub use plan::effective_research_limits;
pub use request::{
    AspectRequest, AspectResearchRequest, DeepResearchRequest, ResearchContext, ResearchPolicy,
    ResearchTask, RuntimeCapabilities, RuntimeCapabilitiesRequest,
};

pub(crate) use plan::{EffectiveAspectPlan, EffectiveResearchPlan, WorkflowValidationContext};
pub(crate) use prompt::{ASPECT_PROMPT_MAX_BYTES, AspectPromptInput};
pub(crate) use request::SUPPORTED_SCHEMA_VERSIONS;
