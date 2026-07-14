//! Configuration boundary for MoeResearch.

pub mod limit;
pub mod loader;
pub mod types;

pub use limit::{ConfigLimit, CountLimit, DurationLimitMs, TokenLimit};
pub use loader::load_config;
pub use types::{
    AgentLimitsConfig, EnabledProviderEnv, GrokReasoningEffort, LimitsConfig, LoggingConfig,
    ModelProviderEndpoint, ModelProviderRegistry, MoeResearchConfig, NetworkConfig,
    NetworkProxyUrl, ResearchLimitsConfig, SearchProviderEndpoint, SearchProviderRegistry,
    WebFetchConfig, WebFetchModelEndpoint,
};
