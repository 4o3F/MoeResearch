use moe_research_net::{Header, JsonNetworkResponse, NetworkRequest, SseEvent};
use serde_json::json;

#[test]
fn safe_header_debug_masks_sensitive_headers() {
    let authorization = format!(
        "{:?}",
        Header {
            name: "authorization".to_owned(),
            value: "Bearer secret".to_owned(),
        }
    );
    let api_key = format!(
        "{:?}",
        Header {
            name: "x-api-key".to_owned(),
            value: "secret".to_owned(),
        }
    );
    let content_type = format!(
        "{:?}",
        Header {
            name: "content-type".to_owned(),
            value: "application/json".to_owned(),
        }
    );

    assert!(authorization.contains("[REDACTED]"));
    assert!(!authorization.contains("Bearer secret"));
    assert!(api_key.contains("[REDACTED]"));
    assert!(!api_key.contains("secret"));
    assert!(content_type.contains("application/json"));
}

#[test]
fn safe_network_request_debug_masks_nested_json_sensitive_fields() {
    let request = NetworkRequest {
        method: "POST".to_owned(),
        url: "https://user:sk-user-secret@example.test/responses?api_key=sk-query-secret"
            .to_owned(),
        headers: vec![Header {
            name: "authorization".to_owned(),
            value: "Bearer header-secret".to_owned(),
        }],
        body: Some(json!({
            "model": "gpt-5.5",
            "authorization": "Bearer body-secret",
            "nested": {
                "access_token": "token-value",
                "safe": "visible"
            },
            "items": [
                { "password": "hidden" },
                { "title": "kept" }
            ]
        })),
        timeout_ms: Some(1_000),
    };
    let rendered = format!("{request:?}");

    for forbidden in [
        "sk-user-secret",
        "sk-query-secret",
        "header-secret",
        "body-secret",
        "token-value",
        "hidden",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "safe Debug leaked `{forbidden}`: {rendered}"
        );
    }
    assert!(rendered.contains("[REDACTED]"));
    assert!(rendered.contains("gpt-5.5"));
    assert!(rendered.contains("visible"));
    assert!(rendered.contains("kept"));
}

#[test]
fn safe_response_debug_keeps_token_usage_metrics() {
    let response = JsonNetworkResponse {
        status: 200,
        headers: Vec::new(),
        body: json!({
            "token_usage": {
                "input_tokens": 3,
                "output_tokens": 5,
                "total_tokens": 8
            },
            "max_tokens": 128,
            "access_token": "secret"
        }),
    };
    let rendered = format!("{response:?}");

    assert!(rendered.contains("\"input_tokens\":3"));
    assert!(rendered.contains("\"output_tokens\":5"));
    assert!(rendered.contains("\"total_tokens\":8"));
    assert!(rendered.contains("\"max_tokens\":128"));
    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("secret"));
}

#[test]
fn safe_response_debug_redacts_json_body() {
    let response = JsonNetworkResponse {
        status: 400,
        headers: Vec::new(),
        body: json!({"error": "bad", "api_key": "secret"}),
    };
    let rendered = format!("{response:?}");

    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("secret"));
    assert!(rendered.contains("bad"));
}

#[test]
fn safe_response_debug_redacts_nested_json_string_body() {
    let response = JsonNetworkResponse {
        status: 400,
        headers: Vec::new(),
        body: json!({
            "error": "{\"api_key\":\"nested-secret\",\"safe\":\"kept\"}"
        }),
    };
    let rendered = format!("{response:?}");

    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("nested-secret"));
    assert!(rendered.contains("kept"));
}

#[test]
fn safe_sse_event_debug_redacts_data() {
    let event = SseEvent {
        event: "response.output_text.delta".to_owned(),
        data: "{\"api_key\":\"stream-secret\",\"delta\":\"ok\"}".to_owned(),
    };
    let rendered = format!("{event:?}");

    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("stream-secret"));
    assert!(rendered.contains("ok"));
}

#[test]
fn safe_header_debug_scrubs_raw_text_credentials_without_truncation() {
    let header = Header {
        name: "x-diagnostic".to_owned(),
        value: "upstream failed Bearer abc123 token=def456 COOKIE: sid=session-secret JWT=jwt-secret safe text".to_owned(),
    };
    let rendered = format!("{header:?}");

    assert!(rendered.contains("Bearer [REDACTED]"));
    assert!(rendered.contains("token=[REDACTED]"));
    assert!(rendered.contains("COOKIE: [REDACTED]"));
    assert!(rendered.contains("JWT=[REDACTED]"));
    assert!(!rendered.contains("abc123"));
    assert!(!rendered.contains("def456"));
    assert!(!rendered.contains("session-secret"));
    assert!(!rendered.contains("jwt-secret"));
    assert!(rendered.contains("safe text"));
}

#[test]
fn safe_debug_does_not_apply_hidden_preview_limit() {
    let body = "a".repeat(4097);
    let header = Header {
        name: "x-diagnostic".to_owned(),
        value: body.clone(),
    };

    assert!(format!("{header:?}").contains(&body));
}
