use schemars::schema_for;

use crate::report::{AspectResearchResult, TokenUsage};
use moe_research_model::{JsonSchemaFormat, ModelResponseFormat};

pub(crate) fn aspect_response_format() -> ModelResponseFormat {
    ModelResponseFormat::JsonSchema(JsonSchemaFormat {
        name: "aspect_research_result_v2".to_owned(),
        strict: true,
        schema: model_output_schema(),
    })
}

fn model_output_schema() -> serde_json::Value {
    let mut schema = serde_json::to_value(schema_for!(AspectResearchResult))
        .expect("AspectResearchResult schema serializes");
    let object = schema
        .as_object_mut()
        .expect("AspectResearchResult schema is an object");
    let properties = object
        .get_mut("properties")
        .and_then(serde_json::Value::as_object_mut)
        .expect("AspectResearchResult schema has properties");
    properties.remove("evidence");

    if let Some(required) = object
        .get_mut("required")
        .and_then(serde_json::Value::as_array_mut)
    {
        required.retain(|field| field.as_str() != Some("evidence"));
    }
    if let Some(definitions) = object
        .get_mut("$defs")
        .and_then(serde_json::Value::as_object_mut)
    {
        definitions.remove("Evidence");
        definitions.remove("SourceType");
    }
    schema
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
