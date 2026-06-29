//! Configuration boundary for MoeResearch.

pub mod limit;
pub mod loader;
pub mod types;

pub use limit::{ConfigLimit, CountLimit, DurationLimitMs, TokenLimit};
pub use loader::load_config;
pub use types::{
    AgentBudgetConfig, BudgetConfig, EnabledProviderEnv, GrokReasoningEffort, LoggingConfig,
    ModelProviderEndpoint, ModelProviderRegistry, MoeResearchConfig, NetworkConfig,
    ResearchBudgetConfig, SearchProviderEndpoint, SearchProviderRegistry,
};
