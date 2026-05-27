use serde_json::Value;

use crate::Header;

const REDACTED: &str = "[REDACTED]";

#[derive(Clone, Copy, Debug, Default)]
pub struct RedactionPolicy;

impl RedactionPolicy {
    pub fn redact_header(&self, name: &str, value: &str) -> String {
        if Self::is_sensitive_name(name) {
            REDACTED.to_owned()
        } else {
            value.to_owned()
        }
    }

    pub fn redact_headers(&self, headers: &[Header]) -> Vec<Header> {
        headers
            .iter()
            .map(|header| Header {
                name: header.name.clone(),
                value: self.redact_header(&header.name, &header.value),
            })
            .collect()
    }

    pub fn redact_json_value(&self, value: &Value) -> Value {
        Self::redact_json_value_inner(value)
    }

    pub fn redact_body_text(&self, text: &str) -> String {
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            return Self::redact_json_value_inner(&value).to_string();
        }

        Self::raw_text_markers()
            .iter()
            .fold(text.to_owned(), |redacted, marker| {
                Self::scrub_marker_value(&redacted, marker)
            })
    }

    fn redact_json_value_inner(value: &Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(key, value)| {
                        let redacted = if Self::is_sensitive_name(key) {
                            Value::String(REDACTED.to_owned())
                        } else {
                            Self::redact_json_value_inner(value)
                        };
                        (key.clone(), redacted)
                    })
                    .collect(),
            ),
            Value::Array(items) => {
                Value::Array(items.iter().map(Self::redact_json_value_inner).collect())
            }
            _ => value.clone(),
        }
    }

    fn is_sensitive_name(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        if Self::is_safe_token_metric_name(&lower) {
            return false;
        }

        Self::sensitive_name_fragments()
            .iter()
            .any(|fragment| lower.contains(fragment))
            || lower == "x-api-key"
    }

    fn is_safe_token_metric_name(name: &str) -> bool {
        matches!(
            name,
            "token_usage"
                | "input_tokens"
                | "output_tokens"
                | "total_tokens"
                | "max_tokens"
                | "max_output_tokens"
        )
    }

    fn sensitive_name_fragments() -> &'static [&'static str] {
        &[
            "authorization",
            "api-key",
            "api_key",
            "apikey",
            "secret",
            "token",
            "password",
            "cookie",
            "session",
            "jwt",
        ]
    }

    fn raw_text_markers() -> &'static [&'static str] {
        &[
            "Bearer ",
            "Basic ",
            "api_key=",
            "apikey=",
            "access_token=",
            "refresh_token=",
            "token=",
            "password=",
            "secret=",
        ]
    }

    fn scrub_marker_value(text: &str, marker: &str) -> String {
        let mut output = String::with_capacity(text.len());
        let mut rest = text;

        while let Some(index) = rest.find(marker) {
            output.push_str(&rest[..index]);
            output.push_str(marker);
            output.push_str(REDACTED);
            let after_marker = &rest[index + marker.len()..];
            let next_delimiter = after_marker
                .find(|ch: char| ch.is_whitespace() || matches!(ch, '&' | ',' | ';' | '"' | '\''))
                .unwrap_or(after_marker.len());
            rest = &after_marker[next_delimiter..];
        }

        output.push_str(rest);
        output
    }
}
