use std::fmt;

use reqwest::Url;
use serde_json::Value;

const REDACTED: &str = "[REDACTED]";

pub(crate) struct SafeJson<'a> {
    value: &'a Value,
}

impl<'a> SafeJson<'a> {
    pub(crate) const fn new(value: &'a Value) -> Self {
        Self { value }
    }
}

impl fmt::Debug for SafeJson<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for SafeJson<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", redact_json_value(self.value))
    }
}

pub(crate) struct SafeText<'a> {
    text: &'a str,
}

impl<'a> SafeText<'a> {
    pub(crate) const fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub(crate) const fn excerpt(self, cap: usize) -> SafeTextExcerpt<'a> {
        SafeTextExcerpt {
            text: self.text,
            cap,
        }
    }
}

impl fmt::Debug for SafeText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string(), f)
    }
}

impl fmt::Display for SafeText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&redact_text(self.text))
    }
}

pub(crate) struct SafeTextExcerpt<'a> {
    text: &'a str,
    cap: usize,
}

impl fmt::Display for SafeTextExcerpt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted = redact_text(self.text);
        f.write_str(&excerpt_for_debug(&redacted, self.cap))
    }
}

pub(crate) struct SafeUrl<'a> {
    raw: &'a str,
}

impl<'a> SafeUrl<'a> {
    pub(crate) const fn new(raw: &'a str) -> Self {
        Self { raw }
    }
}

impl fmt::Debug for SafeUrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string(), f)
    }
}

impl fmt::Display for SafeUrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&redact_url(self.raw))
    }
}

pub(crate) struct SafeHeaderValue<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> SafeHeaderValue<'a> {
    pub(crate) const fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }

    fn redacted(&self) -> String {
        if is_sensitive_name(self.name) {
            REDACTED.to_owned()
        } else {
            redact_text(self.value)
        }
    }
}

impl fmt::Debug for SafeHeaderValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.redacted(), f)
    }
}

pub(crate) struct SafeWireBody<'a> {
    raw: &'a str,
    cap: usize,
}

impl<'a> SafeWireBody<'a> {
    pub(crate) const fn new(raw: &'a str, cap: usize) -> Self {
        Self { raw, cap }
    }
}

impl fmt::Display for SafeWireBody<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted = redact_text(self.raw);
        if self.raw.len() <= self.cap {
            return f.write_str(&redacted);
        }

        let mut cut = redacted.len().min(self.cap);
        while cut > 0 && !redacted.is_char_boundary(cut) {
            cut -= 1;
        }

        let marker = serde_json::json!({
            "__truncated": true,
            "original_bytes": self.raw.len(),
            "head": &redacted[..cut],
        });
        write!(f, "{marker}")
    }
}

fn redact_url(raw: &str) -> String {
    let Ok(mut url) = Url::parse(raw) else {
        return redact_text(raw);
    };

    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

fn redact_text(text: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        return redact_json_value(&value).to_string();
    }

    redact_raw_text(text)
}

fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let value = if is_sensitive_name(key) {
                        Value::String(REDACTED.to_owned())
                    } else {
                        redact_json_value(value)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_json_value).collect()),
        Value::String(text) => Value::String(redact_json_string(text)),
        _ => value.clone(),
    }
}

fn redact_json_string(text: &str) -> String {
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return redact_text(text);
    }
    redact_raw_text(text)
}

fn redact_raw_text(text: &str) -> String {
    let mut redacted = raw_text_markers()
        .iter()
        .fold(text.to_owned(), |redacted, marker| {
            scrub_marker_value_case_insensitive(&redacted, marker)
        });

    for key in sensitive_jsonish_keys() {
        for marker in [
            format!(r#""{key}":""#),
            format!(r#""{key}": ""#),
            format!(r#"\"{key}\":\""#),
            format!(r#"\"{key}\": \""#),
        ] {
            redacted = scrub_marker_value_case_insensitive(&redacted, &marker);
        }
    }

    redacted
}

fn is_sensitive_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if is_safe_token_metric_name(&lower) {
        return false;
    }

    lower == "x-api-key"
        || sensitive_name_fragments()
            .iter()
            .any(|fragment| lower.contains(fragment))
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
        "bearer ",
        "basic ",
        "api_key=",
        "api-key=",
        "apikey=",
        "access_token=",
        "refresh_token=",
        "token=",
        "key=",
        "password=",
        "secret=",
        "authorization=",
        "authorization: ",
        "cookie=",
        "cookie: ",
        "set-cookie: ",
        "session=",
        "session_id=",
        "jwt=",
    ]
}

fn sensitive_jsonish_keys() -> &'static [&'static str] {
    &[
        "authorization",
        "api-key",
        "api_key",
        "apikey",
        "x-api-key",
        "access_token",
        "refresh_token",
        "token",
        "password",
        "secret",
        "cookie",
        "session",
        "jwt",
    ]
}

fn scrub_marker_value_case_insensitive(text: &str, marker: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(index) = remaining.to_ascii_lowercase().find(marker) {
        let (before, after_before) = remaining.split_at(index);
        output.push_str(before);
        output.push_str(&after_before[..marker.len()]);
        output.push_str(REDACTED);

        let after_marker = &after_before[marker.len()..];
        let value_end = after_marker
            .find(|ch: char| ch.is_whitespace() || matches!(ch, '&' | ',' | ';' | '"' | '\'' | ')'))
            .unwrap_or(after_marker.len());
        remaining = &after_marker[value_end..];
    }

    output.push_str(remaining);
    output
}

fn excerpt_for_debug(raw: &str, cap: usize) -> String {
    let body_bytes = raw.len();
    if body_bytes <= cap {
        return raw.to_owned();
    }

    let mut cut = cap;
    while cut > 0 && !raw.is_char_boundary(cut) {
        cut -= 1;
    }

    format!(
        "{}… ({} of {} bytes; enable reqwest_client=trace for full body)",
        &raw[..cut],
        cut,
        body_bytes
    )
}
