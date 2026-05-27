use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use crate::error::{Error, ErrorCode};
use crate::mcp::server::LapisMcpServer;
use crate::orchestrator::agent_loop::AgentRuntimeOutput;
use crate::orchestrator::workflow::{
    aspect_research as run_aspect_research, deep_research as run_deep_research,
};
use crate::schema::mcp::{ToolEnvelope, ToolError, ToolErrorCode, ToolStatus};
use crate::schema::report::{AspectFailure, AspectResearchResult, DeepResearchResult};
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
                self.model_service.inner(),
                self.search_service.inner(),
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
                        error_code = failure.error.code().as_str(),
                        // Full internal Display — only the redacted code +
                        // public message flow into the MCP envelope; the
                        // operator log keeps the detailed message so
                        // schema/budget failures can be diagnosed locally.
                        error_detail = %failure.error,
                        retryable = failure.error.retryable(),
                        status = "failed",
                        "MCP tool failed"
                    );
                    failed_envelope(
                        schema_version,
                        request_id,
                        Some(aspect_id.clone()),
                        &failure.error,
                        Vec::new(),
                    )
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
                self.model_service.inner(),
                self.search_service.inner(),
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
                Err(failure) => {
                    tracing::warn!(
                        request_id = %request_id,
                        tool = "deep_research",
                        error_code = failure.error.code().as_str(),
                        error_detail = %failure.error,
                        retryable = failure.error.retryable(),
                        failed_aspects = failure.failed_aspects.len(),
                        status = "failed",
                        "MCP tool failed"
                    );
                    failed_envelope(
                        schema_version,
                        request_id,
                        None,
                        &failure.error,
                        failure.failed_aspects,
                    )
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

/// Builds a `failed` MCP envelope.
///
/// `aspect_id = Some(_)` is required for single-aspect tool failures (e.g.
/// `aspect_research`) so external clients can pinpoint the failing aspect.
/// Top-level deep-research failures pass `None`.
///
/// `data` and `run_id` are intentionally `None`; the envelope serializes them
/// as JSON `null` to satisfy the contract pinned by `tool_envelope_failed_*`
/// golden tests.
fn failed_envelope<T>(
    schema_version: String,
    request_id: String,
    aspect_id: Option<String>,
    error: &Error,
    failed_aspects: Vec<AspectFailure>,
) -> ToolEnvelope<T> {
    ToolEnvelope {
        schema_version,
        request_id,
        run_id: None,
        status: ToolStatus::Failed,
        data: None,
        error: Some(tool_error_from_error(error, aspect_id, failed_aspects)),
    }
}

#[must_use]
fn tool_error_from_error(
    error: &Error,
    aspect_id: Option<String>,
    failed_aspects: Vec<AspectFailure>,
) -> ToolError {
    ToolError {
        code: tool_error_code(error.code()),
        message: error.public_message(),
        aspect_id,
        retryable: tool_error_retryable(error, &failed_aspects),
        failed_aspects,
    }
}

fn tool_error_retryable(error: &Error, failed_aspects: &[AspectFailure]) -> bool {
    if error.code() == ErrorCode::PartialResult && !failed_aspects.is_empty() {
        failed_aspects.iter().all(|failure| failure.retryable)
    } else {
        error.retryable()
    }
}

fn tool_error_code(code: ErrorCode) -> ToolErrorCode {
    match code {
        ErrorCode::InvalidInput => ToolErrorCode::InvalidInput,
        ErrorCode::UnsupportedSchemaVersion => ToolErrorCode::UnsupportedSchemaVersion,
        ErrorCode::ConfigInvalid => ToolErrorCode::ConfigInvalid,
        ErrorCode::ProviderUnavailable => ToolErrorCode::ProviderUnavailable,
        ErrorCode::NetworkFailed => ToolErrorCode::NetworkFailed,
        ErrorCode::BudgetExceeded => ToolErrorCode::BudgetExceeded,
        ErrorCode::ToolPolicyDenied => ToolErrorCode::ToolPolicyDenied,
        ErrorCode::SchemaValidationFailed => ToolErrorCode::SchemaValidationFailed,
        ErrorCode::Timeout => ToolErrorCode::Timeout,
        ErrorCode::PartialResult => ToolErrorCode::PartialResult,
        ErrorCode::Internal => ToolErrorCode::Internal,
    }
}
