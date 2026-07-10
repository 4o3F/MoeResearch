use moe_research_error::Error;

pub(crate) fn error_message_for_log(error: &Error) -> String {
    let message = match error {
        Error::InvalidInput { message }
        | Error::ConfigInvalid { message }
        | Error::BudgetExceeded { message }
        | Error::ToolPolicyDenied { message }
        | Error::SchemaValidationFailed { message }
        | Error::Timeout { message }
        | Error::PartialResult { message }
        | Error::Internal { message }
        | Error::LoggingInit { message } => message.as_str(),
        Error::ProviderUnavailable { message, .. }
        | Error::NetworkFailed { message }
        | Error::HttpTransport { message, .. }
        | Error::HttpStatus { message, .. } => message.as_str(),
        Error::Json { source } => return json_error_message_for_log(source),
        Error::UnsupportedSchemaVersion { .. }
        | Error::ConfigIo { .. }
        | Error::ConfigParse { .. } => return safe_log_text(&error.public_message()),
    };
    safe_log_text(message)
}

pub(crate) fn json_error_message_for_log(error: &serde_json::Error) -> String {
    let message = error
        .to_string()
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    safe_log_text(&message)
}

pub(crate) fn safe_evidence_id_for_log(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return "<empty>".to_owned();
    }
    if looks_sensitive(trimmed) || !is_generated_evidence_id(trimmed) {
        return "[redacted]".to_owned();
    }
    trimmed.to_owned()
}

pub(crate) fn safe_model_identifier_for_log(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "<empty>".to_owned();
    }
    if !is_safe_log_identifier(trimmed) {
        return "[redacted]".to_owned();
    }
    trimmed.to_owned()
}

fn safe_log_text(value: &str) -> String {
    if looks_sensitive(value) {
        return "[redacted]".to_owned();
    }
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect()
}

fn is_generated_evidence_id(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("ev-") else {
        return false;
    };
    let Some((search_index, evidence_index)) = rest.split_once('-') else {
        return false;
    };
    !search_index.is_empty()
        && !evidence_index.is_empty()
        && search_index
            .chars()
            .all(|character| character.is_ascii_digit())
        && evidence_index
            .chars()
            .all(|character| character.is_ascii_digit())
}

fn is_safe_log_identifier(value: &str) -> bool {
    value.len() <= 128
        && !looks_sensitive(value)
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | ':' | '.')
        })
}

fn looks_sensitive(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("authorization")
        || value.contains("bearer")
        || value.contains("cookie")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.contains("access_token")
        || value.contains("jwt")
        || value.starts_with("sk-")
        || value.contains("://")
        || value.contains('/')
        || value.contains('\\')
}
