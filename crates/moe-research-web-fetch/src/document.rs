use dom_smoothie::{Config, Readability, TextMode};
use encoding_rs::{Encoding, UTF_8};

use crate::WebFetchSoftError;

pub(crate) struct ConvertedDocument {
    pub(crate) title: String,
    pub(crate) markdown: String,
}

pub(crate) fn convert_document(
    body: &[u8],
    content_type: &str,
    final_url: &str,
) -> Result<ConvertedDocument, WebFetchSoftError> {
    let text = decode(body, charset_label(content_type));
    let media_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    let (title, markdown) = match media_type.as_str() {
        "text/html" | "application/xhtml+xml" | "" => html_to_markdown(&text, final_url)?,
        "text/plain" | "text/markdown" | "text/x-markdown" => (String::new(), text),
        _ => {
            return Err(WebFetchSoftError::new(
                "unsupported_content_type",
                false,
                "the document content type is not supported",
            ));
        }
    };

    if markdown.trim().is_empty() {
        return Err(WebFetchSoftError::new(
            "empty_document",
            false,
            "the document did not contain readable text",
        ));
    }
    Ok(ConvertedDocument { title, markdown })
}

fn html_to_markdown(html: &str, final_url: &str) -> Result<(String, String), WebFetchSoftError> {
    let config = Config {
        text_mode: TextMode::Markdown,
        ..Config::default()
    };
    let readability = Readability::new(html, Some(final_url), Some(config));
    if let Ok(mut readability) = readability
        && let Ok(article) = readability.parse()
    {
        let markdown = article.text_content.to_string();
        if !markdown.trim().is_empty() {
            return Ok((article.title, markdown));
        }
    }

    htmd::convert(html)
        .map(|markdown| (String::new(), markdown))
        .map_err(|_| {
            WebFetchSoftError::new(
                "document_conversion_failed",
                false,
                "the document could not be converted to text",
            )
        })
}

fn charset_label(content_type: &str) -> Option<&str> {
    content_type.split(';').skip(1).find_map(|parameter| {
        let (name, value) = parameter.trim().split_once('=')?;
        name.eq_ignore_ascii_case("charset")
            .then_some(value.trim().trim_matches('"'))
    })
}

fn decode(body: &[u8], declared: Option<&str>) -> String {
    let (encoding, bom_len) = Encoding::for_bom(body).unwrap_or_else(|| {
        (
            declared
                .and_then(|label| Encoding::for_label(label.as_bytes()))
                .unwrap_or(UTF_8),
            0,
        )
    });
    let (decoded, _, _) = encoding.decode(&body[bom_len..]);
    decoded.into_owned()
}
