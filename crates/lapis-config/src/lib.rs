//! Configuration boundary for Lapis.

pub mod limit;
pub mod loader;
pub mod types;

pub use limit::{ConfigLimit, CountLimit, DurationLimitMs, TokenLimit};
pub use loader::load_config;
pub use types::{
    AgentBudgetConfig, BudgetConfig, LapisConfig, LoggingConfig, ModelProviderEndpoint,
    ModelProviderRegistry, NetworkConfig, ResearchBudgetConfig, SearchProviderEndpoint,
    SearchProviderRegistry,
};
