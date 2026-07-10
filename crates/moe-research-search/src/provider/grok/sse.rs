use serde_json::Value;
use snafu::ResultExt;

use moe_research_error::{Error, JsonSnafu, Result};
use moe_research_net::{SseEvent, SseNetworkStream};

pub(super) async fn assemble_grok_sse(stream: &mut SseNetworkStream) -> Result<Value> {
    while let Some(event) = stream.next_event().await? {
        if event.data == "[DONE]" {
            break;
        }
        let value: Value = serde_json::from_str(&event.data).context(JsonSnafu)?;
        match sse_event_type(&event, &value) {
            Some("response.completed") => {
                return value.get("response").cloned().ok_or_else(|| {
                    Error::SchemaValidationFailed {
                        message: "grok response.completed missing response".to_owned(),
                    }
                });
            }
            Some("response.failed" | "response.incomplete") => {
                return Err(Error::ProviderUnavailable {
                    provider: "grok".to_owned(),
                    message: "SSE stream ended with terminal failure".to_owned(),
                    retryable: true,
                });
            }
            Some("error") => {
                return Err(Error::ProviderUnavailable {
                    provider: "grok".to_owned(),
                    message: "SSE stream returned error event".to_owned(),
                    retryable: true,
                });
            }
            _ => {}
        }
    }

    Err(Error::SchemaValidationFailed {
        message: "grok SSE ended before terminal response event".to_owned(),
    })
}

fn sse_event_type<'a>(event: &'a SseEvent, data: &'a Value) -> Option<&'a str> {
    if !event.event.is_empty() && event.event != "message" {
        Some(event.event.as_str())
    } else {
        data.get("type").and_then(Value::as_str)
    }
}
