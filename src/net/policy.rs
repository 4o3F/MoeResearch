use crate::schema::common::Header;

const REDACTED: &str = "[REDACTED]";

pub fn redact_header(name: &str, value: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower.contains("authorization")
        || lower.contains("api-key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower == "x-api-key"
    {
        REDACTED.to_owned()
    } else {
        value.to_owned()
    }
}

pub fn redact_headers(headers: &[Header]) -> Vec<Header> {
    headers
        .iter()
        .map(|header| Header {
            name: header.name.clone(),
            value: redact_header(&header.name, &header.value),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_sensitive_headers() {
        assert_eq!(redact_header("Authorization", "Bearer secret"), REDACTED);
        assert_eq!(redact_header("x-api-key", "secret"), REDACTED);
        assert_eq!(
            redact_header("content-type", "application/json"),
            "application/json"
        );
    }
}
