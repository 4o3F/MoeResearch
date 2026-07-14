use std::sync::Arc;

use rmcp::service::serve_server;

use moe_research_error::{Error, Result};
use moe_research_model::ModelService;
use moe_research_search::SearchService;
use moe_research_web_fetch::WebFetchService;
use moe_research_workflow::BudgetConfig;

#[derive(Clone)]
pub struct MoeResearchMcpServer {
    pub(crate) model_service: Arc<ModelService>,
    pub(crate) search_service: Arc<SearchService>,
    pub(crate) web_fetch_service: Arc<WebFetchService>,
    pub(crate) budget_config: BudgetConfig,
}

impl MoeResearchMcpServer {
    #[must_use]
    pub fn new(
        model_service: ModelService,
        search_service: SearchService,
        web_fetch_service: WebFetchService,
        budget_config: BudgetConfig,
    ) -> Self {
        Self {
            model_service: Arc::new(model_service),
            search_service: Arc::new(search_service),
            web_fetch_service: Arc::new(web_fetch_service),
            budget_config,
        }
    }
}

pub async fn serve_stdio(
    model_service: ModelService,
    search_service: SearchService,
    web_fetch_service: WebFetchService,
    budget_config: BudgetConfig,
) -> Result<()> {
    let server = MoeResearchMcpServer::new(
        model_service,
        search_service,
        web_fetch_service,
        budget_config,
    );
    let running = serve_server(server, rmcp::transport::io::stdio())
        .await
        .map_err(|error| Error::Internal {
            message: format!("MCP server initialization failed: {error}"),
        })?;

    running.waiting().await.map_err(|error| Error::Internal {
        message: format!("MCP server task failed: {error}"),
    })?;

    Ok(())
}
