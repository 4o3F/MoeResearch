use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use crate::error::Error;
use crate::mcp::server::LapisMcpServer;
use crate::orchestrator::agent_loop::AgentRuntimeOutput;
use crate::orchestrator::workflow::{
    aspect_research as run_aspect_research, deep_research as run_deep_research,
};
use crate::schema::mcp::{ToolEnvelope, ToolStatus};
use crate::schema::report::{AspectResearchResult, DeepResearchResult};
use crate::schema::research::{AspectResearchRequest, DeepResearchRequest};

#[tool_router(server_handler)]
impl LapisMcpServer {
    #[tool(
        description = "Run one research aspect and return a ToolEnvelope containing an AspectResearchResult."
    )]
    pub async fn aspect_research(
        &self,
        Parameters(request): Parameters<AspectResearchRequest>,
    ) -> Json<ToolEnvelope<AspectResearchResult>> {
        let schema_version = request.schema_version.clone();
        let request_id = request.request_id.clone();
        let aspect_id = request.task.aspect.aspect_id.clone();
        tracing::info!(
            request_id = %request_id,
            aspect_id = %aspect_id,
            tool = "aspect_research",
            "MCP tool started"
        );

        Json(
            match run_aspect_research(
                request,
                &self.model_service,
                &self.search_service,
                &self.budget_config,
            )
            .await
            {
                Ok(output) => {
                    tracing::info!(
                        request_id = %request_id,
                        aspect_id = %aspect_id,
                        tool = "aspect_research",
                        status = "ok",
                        "MCP tool completed"
                    );
                    aspect_success_envelope(schema_version, request_id, output)
                }
                Err(failure) => {
                    tracing::warn!(
                        request_id = %request_id,
                        aspect_id = %aspect_id,
                        tool = "aspect_research",
                        error_code = ?failure.error.code(),
                        retryable = failure.error.to_tool_error().retryable,
                        status = "failed",
                        "MCP tool failed"
                    );
                    failed_envelope(schema_version, request_id, &failure.error)
                }
            },
        )
    }

    #[tool(
        description = "Run a deep research plan and return a ToolEnvelope containing a DeepResearchResult."
    )]
    pub async fn deep_research(
        &self,
        Parameters(request): Parameters<DeepResearchRequest>,
    ) -> Json<ToolEnvelope<DeepResearchResult>> {
        let schema_version = request.schema_version.clone();
        let request_id = request.request_id.clone();
        tracing::info!(
            request_id = %request_id,
            tool = "deep_research",
            "MCP tool started"
        );

        Json(
            match run_deep_research(
                request,
                &self.model_service,
                &self.search_service,
                &self.budget_config,
            )
            .await
            {
                Ok(result) => {
                    tracing::info!(
                        request_id = %request_id,
                        run_id = %result.run_id,
                        tool = "deep_research",
                        status = if result.failed_aspects.is_empty() { "ok" } else { "partial" },
                        "MCP tool completed"
                    );
                    deep_success_envelope(schema_version, request_id, result)
                }
                Err(error) => {
                    tracing::warn!(
                        request_id = %request_id,
                        tool = "deep_research",
                        error_code = ?error.code(),
                        retryable = error.to_tool_error().retryable,
                        status = "failed",
                        "MCP tool failed"
                    );
                    failed_envelope(schema_version, request_id, &error)
                }
            },
        )
    }
}

fn aspect_success_envelope(
    schema_version: String,
    request_id: String,
    output: AgentRuntimeOutput,
) -> ToolEnvelope<AspectResearchResult> {
    ToolEnvelope {
        schema_version,
        request_id,
        run_id: None,
        status: ToolStatus::Ok,
        data: Some(output.result),
        error: None,
    }
}

fn deep_success_envelope(
    schema_version: String,
    request_id: String,
    result: DeepResearchResult,
) -> ToolEnvelope<DeepResearchResult> {
    let run_id = result.run_id.clone();
    let status = if result.failed_aspects.is_empty() {
        ToolStatus::Ok
    } else {
        ToolStatus::Partial
    };

    ToolEnvelope {
        schema_version,
        request_id,
        run_id: Some(run_id),
        status,
        data: Some(result),
        error: None,
    }
}

fn failed_envelope<T>(
    schema_version: String,
    request_id: String,
    error: &Error,
) -> ToolEnvelope<T> {
    ToolEnvelope {
        schema_version,
        request_id,
        run_id: None,
        status: ToolStatus::Failed,
        data: None,
        error: Some(error.to_tool_error()),
    }
}
