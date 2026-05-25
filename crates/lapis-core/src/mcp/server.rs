use std::sync::Arc;

use rmcp::service::serve_server;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::schema::budget::BudgetConfig;
use crate::search::service::SearchService;

#[derive(Clone)]
pub struct LapisMcpServer {
    pub(crate) model_service: Arc<ModelService>,
    pub(crate) search_service: Arc<SearchService>,
    pub(crate) budget_config: BudgetConfig,
}

impl LapisMcpServer {
    pub fn new(
        model_service: ModelService,
        search_service: SearchService,
        budget_config: BudgetConfig,
    ) -> Self {
        Self {
            model_service: Arc::new(model_service),
            search_service: Arc::new(search_service),
            budget_config,
        }
    }
}

pub async fn serve_stdio(
    model_service: ModelService,
    search_service: SearchService,
    budget_config: BudgetConfig,
) -> Result<()> {
    let server = LapisMcpServer::new(model_service, search_service, budget_config);
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
