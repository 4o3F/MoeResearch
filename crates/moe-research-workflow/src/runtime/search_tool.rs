use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use moe_research_error::{Error, Result};
use moe_research_model::{ModelTool, ModelToolCall};

use crate::error_log_safe::json_error_message_for_log;
use crate::policy::{SearchCategory, SearchContentLevel, SearchDepth, SearchRecency};
use crate::research::AspectRequest;

pub const SEARCH_TOOL_NAME: &str = "search";

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchToolArgs {
    pub query: String,
    #[schemars(schema_with = "crate::limit::optional_non_negative_integer_schema")]
    pub max_results: Option<usize>,
    pub depth: Option<SearchDepth>,
    pub content_level: Option<SearchContentLevel>,
    pub recency: Option<SearchRecency>,
    pub category: Option<SearchCategory>,
}

#[derive(Clone, Debug)]
pub struct ToolPolicyGuard {
    search_allowed: bool,
}

impl ToolPolicyGuard {
    pub fn new(aspect: &AspectRequest) -> Self {
        Self {
            search_allowed: aspect.tools.iter().any(|tool| tool.0 == SEARCH_TOOL_NAME),
        }
    }

    /// Returns the subset of model-facing tools the current aspect's policy
    /// allows.
    ///
    /// The orchestrator uses this to drive `ModelRequest.tools`: aspects with
    /// `tools = []` get an empty tools list (no tool calls possible),
    /// while aspects that permit search get exactly the search tool. This is
    /// strictly tighter than always advertising the full tool catalogue and
    /// closes the gap where a model could call a denied tool just because it
    /// was visible in the request.
    #[must_use]
    pub fn allowed_model_tools(&self) -> Vec<ModelTool> {
        let mut tools = Vec::new();
        if self.search_allowed {
            tools.push(search_model_tool());
        }
        tools
    }

    pub fn validate_search_call(&self, call: &ModelToolCall) -> Result<SearchToolArgs> {
        if call.name != SEARCH_TOOL_NAME {
            return Err(Error::ToolPolicyDenied {
                message: "model requested an unknown logical tool".to_owned(),
            });
        }

        if !self.search_allowed {
            return Err(Error::ToolPolicyDenied {
                message: "aspect is not allowed to use search".to_owned(),
            });
        }

        let args: SearchToolArgs =
            serde_json::from_value(call.arguments.clone()).map_err(|error| {
                Error::ToolPolicyDenied {
                    message: format!(
                        "search tool arguments are malformed: {}",
                        json_error_message_for_log(&error)
                    ),
                }
            })?;

        if args.query.trim().is_empty() {
            return Err(Error::ToolPolicyDenied {
                message: "search query must not be empty".to_owned(),
            });
        }

        if args.max_results == Some(0) {
            return Err(Error::ToolPolicyDenied {
                message: "search max_results must be greater than zero when provided".to_owned(),
            });
        }

        Ok(args)
    }
}

pub fn search_model_tool() -> ModelTool {
    ModelTool {
        name: SEARCH_TOOL_NAME.to_owned(),
        description: "Search trusted external sources for evidence relevant to the aspect."
            .to_owned(),
        input_schema: serde_json::to_value(schema_for!(SearchToolArgs))
            .expect("search tool schema serializes to JSON"),
    }
}
