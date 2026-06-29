use std::ffi::{OsStr, OsString};
use std::path::Path;

#[must_use]
pub fn format_command(command: &Path, argv: &[OsString]) -> String {
    let mut parts = Vec::with_capacity(argv.len() + 1);
    parts.push(shell_escape(command.as_os_str()));
    parts.extend(argv.iter().map(OsString::as_os_str).map(shell_escape));
    parts.join(" ")
}

#[must_use]
pub fn shell_escape(value: &OsStr) -> String {
    let value = value.to_string_lossy();
    if value.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/' | b':')
    }) {
        value.into_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
