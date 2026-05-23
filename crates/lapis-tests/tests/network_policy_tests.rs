use lapis_core::net::policy::RedactionPolicy;
use serde_json::json;

#[test]
fn redaction_policy_masks_sensitive_headers() {
    let policy = RedactionPolicy;

    assert_eq!(
        policy.redact_header("authorization", "Bearer secret"),
        "[REDACTED]"
    );
    assert_eq!(policy.redact_header("x-api-key", "secret"), "[REDACTED]");
    assert_eq!(
        policy.redact_header("content-type", "application/json"),
        "application/json"
    );
}

#[test]
fn redaction_policy_masks_nested_json_sensitive_fields() {
    let policy = RedactionPolicy;
    let value = json!({
        "model": "gpt-5.5",
        "authorization": "Bearer secret",
        "nested": {
            "access_token": "token-value",
            "safe": "visible"
        },
        "items": [
            { "password": "hidden" },
            { "title": "kept" }
        ]
    });

    assert_eq!(
        policy.redact_json_value(&value),
        json!({
            "model": "gpt-5.5",
            "authorization": "[REDACTED]",
            "nested": {
                "access_token": "[REDACTED]",
                "safe": "visible"
            },
            "items": [
                { "password": "[REDACTED]" },
                { "title": "kept" }
            ]
        })
    );
}

#[test]
fn redaction_policy_redacts_json_text_body() {
    let policy = RedactionPolicy;
    let redacted = policy.redact_body_text(r#"{"error":"bad","api_key":"secret"}"#);

    assert!(redacted.contains("[REDACTED]"));
    assert!(!redacted.contains("secret"));
}

#[test]
fn redaction_policy_scrubs_raw_text_credentials_without_truncation() {
    let policy = RedactionPolicy;
    let redacted = policy.redact_body_text("upstream failed Bearer abc123 token=def456 safe text");

    assert!(redacted.contains("Bearer [REDACTED]"));
    assert!(redacted.contains("token=[REDACTED]"));
    assert!(!redacted.contains("abc123"));
    assert!(!redacted.contains("def456"));
    assert!(redacted.contains("safe text"));
}

#[test]
fn redaction_policy_does_not_apply_hidden_preview_limit() {
    let policy = RedactionPolicy;
    let body = "a".repeat(4097);

    assert_eq!(policy.redact_body_text(&body), body);
}
