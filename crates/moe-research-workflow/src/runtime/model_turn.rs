use schemars::schema_for;

use crate::report::{AspectResearchResult, TokenUsage};
use moe_research_model::{JsonSchemaFormat, ModelResponseFormat};

pub(crate) fn aspect_response_format() -> ModelResponseFormat {
    ModelResponseFormat::JsonSchema(JsonSchemaFormat {
        name: "aspect_research_result_v1".to_owned(),
        strict: true,
        schema: serde_json::to_value(schema_for!(AspectResearchResult))
            .expect("AspectResearchResult schema serializes"),
    })
}

pub(crate) fn add_token_usage(total: &mut Option<TokenUsage>, delta: Option<TokenUsage>) {
    let Some(delta) = delta else {
        return;
    };
    let usage = total.get_or_insert_with(TokenUsage::zero);
    usage.input_tokens = sum_optional(usage.input_tokens, delta.input_tokens);
    usage.output_tokens = sum_optional(usage.output_tokens, delta.output_tokens);
    usage.total_tokens = sum_optional(usage.total_tokens, delta.total_tokens);
}

pub(crate) fn sum_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.saturating_add(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}
