use async_trait::async_trait;
use moe_research_error::Result;

use crate::{
    ResolvedSearchIntent, SearchIntent, SearchIntentConstraints, SearchRequest, SearchResponse,
};

pub mod exa;
pub mod grok;
pub mod tavily;

pub use exa::ExaSearchProvider;
pub use grok::{GrokReasoningEffort, GrokSearchProvider};
pub use tavily::TavilySearchProvider;

#[async_trait]
pub trait SearchProvider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Resolves a provider-neutral model intent for this provider only.
    ///
    /// Concrete providers override this to report their actual enforcement
    /// semantics. The default keeps test providers usable without duplicating
    /// provider-specific behavior in the workflow crate.
    fn resolve_intent(
        &self,
        base: SearchRequest,
        intent: &SearchIntent,
        constraints: &SearchIntentConstraints,
    ) -> Result<ResolvedSearchIntent> {
        let prepared = constraints.prepare(base, intent)?;
        let resolution = intent.default_resolution();
        Ok(ResolvedSearchIntent {
            request: prepared.request,
            resolution,
        })
    }

    async fn search(&self, request: SearchRequest) -> Result<SearchResponse>;
}
