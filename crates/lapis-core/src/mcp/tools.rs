use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use crate::error::Error;
use crate::mcp::server::LapisMcpServer;
use crate::orchestrator::workflow::{
    aspect_research as run_aspect_research, deep_research as run_deep_research,
};
use crate::schema::mcp::{ToolEnvelope, ToolStatus, Warning};
use crate::schema::report::{AspectResearchResult, DeepResearchResult, PartialTrace, TraceSummary};
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

        Json(
            match run_aspect_research(
                request,
                &self.model_service,
                &self.search_service,
                &self.budget_config,
            )
            .await
            {
                Ok(result) => aspect_success_envelope(schema_version, request_id, result),
                Err(failure) => failed_envelope(
                    schema_version,
                    request_id,
                    &failure.error,
                    failure.partial_trace,
                ),
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

        Json(
            match run_deep_research(
                request,
                &self.model_service,
                &self.search_service,
                &self.budget_config,
            )
            .await
            {
                Ok(result) => deep_success_envelope(schema_version, request_id, result),
                Err(error) => failed_envelope(schema_version, request_id, &error, None),
            },
        )
    }
}

fn aspect_success_envelope(
    schema_version: String,
    request_id: String,
    result: AspectResearchResult,
) -> ToolEnvelope<AspectResearchResult> {
    let trace_summary = result.trace_summary.clone();
    ToolEnvelope {
        schema_version,
        request_id,
        run_id: non_empty_trace_id(&trace_summary),
        status: ToolStatus::Ok,
        data: Some(result),
        warnings: Vec::new(),
        error: None,
        trace_summary: Some(trace_summary),
        partial_trace: None,
    }
}

fn deep_success_envelope(
    schema_version: String,
    request_id: String,
    result: DeepResearchResult,
) -> ToolEnvelope<DeepResearchResult> {
    let run_id = result.run_id.clone();
    let trace_summary = result.trace_summary.clone();
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
        warnings: Vec::new(),
        error: None,
        trace_summary: Some(trace_summary),
        partial_trace: None,
    }
}

fn failed_envelope<T>(
    schema_version: String,
    request_id: String,
    error: &Error,
    partial_trace: Option<PartialTrace>,
) -> ToolEnvelope<T> {
    let trace_summary = partial_trace
        .as_ref()
        .map(|partial| partial.trace_summary.clone());
    let run_id = trace_summary.as_ref().and_then(non_empty_trace_id);
    ToolEnvelope {
        schema_version,
        request_id,
        run_id,
        status: ToolStatus::Failed,
        data: None,
        warnings: Vec::<Warning>::new(),
        error: Some(error.to_tool_error()),
        trace_summary,
        partial_trace,
    }
}

fn non_empty_trace_id(trace_summary: &TraceSummary) -> Option<String> {
    if trace_summary.trace_id.is_empty() {
        None
    } else {
        Some(trace_summary.trace_id.clone())
    }
}
