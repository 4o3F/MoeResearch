use moe_research_net::reqwest_client::ReqwestNetworkClient;
use moe_research_net::{
    DocumentNetworkOutcome, DocumentNetworkRejection, Header, JsonNetworkResponse, NetworkClient,
    NetworkRequest, SseEvent,
};
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
        inactivity_timeout_ms: Some(1_000),
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

fn document_request(url: &str) -> NetworkRequest {
    NetworkRequest {
        method: "GET".to_owned(),
        url: url.to_owned(),
        headers: vec![
            Header {
                name: "accept".to_owned(),
                value: "text/html,text/plain,text/markdown,application/xhtml+xml".to_owned(),
            },
            Header {
                name: "accept-encoding".to_owned(),
                value: "identity".to_owned(),
            },
        ],
        body: None,
        inactivity_timeout_ms: Some(1_000),
    }
}

#[tokio::test]
async fn document_transport_rejects_unsafe_schemes_credentials_and_addresses() {
    let client =
        ReqwestNetworkClient::new(1_000, 0, 0, "moeresearch-test", None).expect("network client");
    let cases = [
        ("http://example.com", DocumentNetworkRejection::UnsafeScheme),
        (
            "https://user:secret@example.com",
            DocumentNetworkRejection::CredentialsPresent,
        ),
        (
            "https://127.0.0.1/",
            DocumentNetworkRejection::UnsafeResolvedAddress,
        ),
        (
            "https://10.0.0.1/",
            DocumentNetworkRejection::UnsafeResolvedAddress,
        ),
        (
            "https://192.0.2.1/",
            DocumentNetworkRejection::UnsafeResolvedAddress,
        ),
        (
            "https://[::1]/",
            DocumentNetworkRejection::UnsafeResolvedAddress,
        ),
        (
            "https://[::ffff:127.0.0.1]/",
            DocumentNetworkRejection::UnsafeResolvedAddress,
        ),
    ];

    for (url, expected) in cases {
        let outcome = client
            .send_document(document_request(url))
            .await
            .expect("typed rejection");
        assert_eq!(outcome, DocumentNetworkOutcome::Rejected(expected), "{url}");
    }
}

#[tokio::test]
async fn document_transport_requires_get_without_body_and_restricted_headers() {
    let client =
        ReqwestNetworkClient::new(1_000, 0, 0, "moeresearch-test", None).expect("network client");

    let mut post = document_request("https://example.com");
    post.method = "POST".to_owned();
    assert!(client.send_document(post).await.is_err());

    let mut with_body = document_request("https://example.com");
    with_body.body = Some(json!({"prompt": "ignored"}));
    assert!(client.send_document(with_body).await.is_err());

    let mut with_authorization = document_request("https://example.com");
    with_authorization.headers.push(Header {
        name: "authorization".to_owned(),
        value: "Bearer secret".to_owned(),
    });
    assert!(client.send_document(with_authorization).await.is_err());
}
