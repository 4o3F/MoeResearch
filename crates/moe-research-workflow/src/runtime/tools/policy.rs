use moe_research_error::{Error, Result};
use moe_research_model::{ModelTool, ModelToolCall};

use crate::research::AspectRequest;

use super::search::{SEARCH_TOOL_NAME, SearchToolArgs, search_model_tool, validate_search_call};
use super::web_fetch::{
    WEB_FETCH_TOOL_NAME, WebFetchToolArgs, validate_web_fetch_call, web_fetch_model_tool,
};

pub const SUPPORTED_ASPECT_TOOLS: &[&str] = &[SEARCH_TOOL_NAME, WEB_FETCH_TOOL_NAME];

#[derive(Clone, Debug)]
pub enum ValidatedToolCall {
    Search(SearchToolArgs),
    WebFetch(WebFetchToolArgs),
}

#[derive(Clone, Debug)]
pub struct ToolPolicyGuard {
    search_allowed: bool,
    web_fetch_allowed: bool,
}

impl ToolPolicyGuard {
    pub fn new(aspect: &AspectRequest) -> Self {
        Self {
            search_allowed: aspect.tools.iter().any(|tool| tool.0 == SEARCH_TOOL_NAME),
            web_fetch_allowed: aspect
                .tools
                .iter()
                .any(|tool| tool.0 == WEB_FETCH_TOOL_NAME),
        }
    }

    #[must_use]
    pub fn allowed_model_tools(&self) -> Vec<ModelTool> {
        let mut tools = Vec::new();
        if self.search_allowed {
            tools.push(search_model_tool());
        }
        if self.web_fetch_allowed {
            tools.push(web_fetch_model_tool());
        }
        tools
    }

    pub fn validate_call(&self, call: &ModelToolCall) -> Result<ValidatedToolCall> {
        match call.name.as_str() {
            SEARCH_TOOL_NAME if self.search_allowed => {
                validate_search_call(call).map(ValidatedToolCall::Search)
            }
            WEB_FETCH_TOOL_NAME if self.web_fetch_allowed => {
                validate_web_fetch_call(call).map(ValidatedToolCall::WebFetch)
            }
            SEARCH_TOOL_NAME => Err(tool_not_allowed(SEARCH_TOOL_NAME)),
            WEB_FETCH_TOOL_NAME => Err(tool_not_allowed(WEB_FETCH_TOOL_NAME)),
            _ => Err(Error::ToolPolicyDenied {
                message: "model requested an unknown logical tool".to_owned(),
                public: false,
            }),
        }
    }
}

pub(super) fn tool_args_error(tool: &'static str, key: &'static str) -> Error {
    Error::ToolPolicyDenied {
        message: format!("invalid {tool} tool arguments [branch=model_tool_args key={key}]"),
        public: true,
    }
}

fn tool_not_allowed(tool: &'static str) -> Error {
    Error::ToolPolicyDenied {
        message: format!("aspect is not allowed to use {tool}"),
        public: false,
    }
}
