use std::collections::BTreeSet;

use crate::error::{Error, Result};
use crate::model::service::ModelService;
use crate::orchestrator::agent_loop::{AgentRuntime, AgentRuntimeOutput};
use crate::orchestrator::tool_policy::SEARCH_TOOL_NAME;
use crate::schema::common::AspectResearchRequest;
use crate::schema::report::AspectResearchResult;
use crate::search::service::SearchService;

const SUPPORTED_SCHEMA_VERSIONS: &[&str] = &["m4", "1", "1.0"];

pub async fn aspect_research(
    request: AspectResearchRequest,
    model_service: &ModelService,
    search_service: &SearchService,
) -> Result<AspectResearchResult> {
    validate_request(&request)?;
    AgentRuntime::new(model_service, search_service, &request)
        .run()
        .await
        .map(AgentRuntimeOutput::into_result)
}

fn validate_request(request: &AspectResearchRequest) -> Result<()> {
    require_non_empty("schema_version", &request.schema_version)?;
    require_non_empty("request_id", &request.request_id)?;
    require_non_empty("aspect.aspect_id", &request.aspect.aspect_id)?;
    require_non_empty("aspect.name", &request.aspect.name)?;
    require_non_empty(
        "aspect.research_question",
        &request.aspect.research_question,
    )?;

    if !SUPPORTED_SCHEMA_VERSIONS.contains(&request.schema_version.as_str()) {
        return Err(Error::SchemaValidationFailed {
            message: format!("unsupported schema version: {}", request.schema_version),
        });
    }

    if request.search_policy.max_results_per_query == 0 {
        return Err(Error::InvalidInput {
            message: "search_policy.max_results_per_query must be greater than 0".to_owned(),
        });
    }

    validate_domains(request)?;
    validate_timeout(request)?;
    validate_tools(request)
}

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::InvalidInput {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

fn validate_domains(request: &AspectResearchRequest) -> Result<()> {
    let include = request
        .search_policy
        .include_domains
        .iter()
        .map(|domain| domain.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();

    if let Some(domain) = request
        .search_policy
        .exclude_domains
        .iter()
        .map(|domain| domain.to_ascii_lowercase())
        .find(|domain| include.contains(domain))
    {
        return Err(Error::InvalidInput {
            message: format!("domain appears in both include and exclude lists: {domain}"),
        });
    }

    Ok(())
}

fn validate_timeout(request: &AspectResearchRequest) -> Result<()> {
    if let Some(timeout_ms) = request.execution_policy.timeout_ms
        && timeout_ms > request.budget.timeout_ms
    {
        return Err(Error::BudgetExceeded {
            message: "execution timeout must not exceed agent budget timeout".to_owned(),
        });
    }
    Ok(())
}

fn validate_tools(request: &AspectResearchRequest) -> Result<()> {
    if let Some(tool) = request
        .aspect
        .allowed_tools
        .iter()
        .find(|tool| tool.0 != SEARCH_TOOL_NAME)
    {
        return Err(Error::ToolPolicyDenied {
            message: format!("unsupported tool for aspect runtime: {}", tool.0),
        });
    }
    Ok(())
}
