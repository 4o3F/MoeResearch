use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const UNKNOWN: &str = "unknown";

fn main() {
    for path in ["build.rs", "Cargo.toml", "src"] {
        println!("cargo:rerun-if-changed={path}");
    }
    for variable in [
        "MOERESEARCH_LOCAL_VERSION",
        "MOERESEARCH_GIT_COMMIT",
        "MOERESEARCH_GIT_DIRTY",
        "GITHUB_SHA",
        "PROFILE",
        "TARGET",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }

    let workspace_root = workspace_root();
    register_git_inputs(&workspace_root);

    emit(
        "MOERESEARCH_BUILD_LOCAL_VERSION",
        &resolve_local_version(&workspace_root),
    );
    emit(
        "MOERESEARCH_BUILD_GIT_COMMIT",
        &resolve_git_commit(&workspace_root),
    );
    emit(
        "MOERESEARCH_BUILD_GIT_DIRTY",
        &resolve_git_dirty(&workspace_root),
    );
    emit(
        "MOERESEARCH_BUILD_PROFILE",
        &environment_value("PROFILE").unwrap_or_else(|| UNKNOWN.to_owned()),
    );
    emit(
        "MOERESEARCH_BUILD_TARGET",
        &environment_value("TARGET").unwrap_or_else(|| UNKNOWN.to_owned()),
    );
}

fn workspace_root() -> PathBuf {
    let root = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("../..");

    root.canonicalize().unwrap_or(root)
}

fn register_git_inputs(root: &Path) {
    for input in ["HEAD", "packed-refs", "refs/tags"] {
        if let Some(path) = git_path(root, input) {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    if let Some(reference) = git_stdout(root, &["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git_path(root, &reference)
    {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn git_path(root: &Path, input: &str) -> Option<PathBuf> {
    let path = PathBuf::from(git_stdout(root, &["rev-parse", "--git-path", input])?);
    Some(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

fn resolve_local_version(root: &Path) -> String {
    environment_value("MOERESEARCH_LOCAL_VERSION")
        .or_else(|| git_stdout(root, &["describe", "--tags", "--always"]))
        .unwrap_or_else(|| UNKNOWN.to_owned())
}

fn resolve_git_commit(root: &Path) -> String {
    environment_commit("MOERESEARCH_GIT_COMMIT")
        .or_else(|| {
            git_stdout(root, &["rev-parse", "--verify", "HEAD"])
                .and_then(|value| commit_value(&value))
        })
        .or_else(|| environment_commit("GITHUB_SHA"))
        .unwrap_or_else(|| UNKNOWN.to_owned())
}

fn resolve_git_dirty(root: &Path) -> String {
    environment_dirty("MOERESEARCH_GIT_DIRTY").unwrap_or_else(|| match git_status_dirty(root) {
        Some(true) => "true".to_owned(),
        Some(false) => "false".to_owned(),
        None => UNKNOWN.to_owned(),
    })
}

fn environment_value(name: &str) -> Option<String> {
    env::var(name).ok().and_then(|value| single_line(&value))
}

fn environment_commit(name: &str) -> Option<String> {
    environment_value(name).and_then(|value| commit_value(&value))
}

fn environment_dirty(name: &str) -> Option<String> {
    let value = environment_value(name)?;
    matches!(value.as_str(), "true" | "false" | UNKNOWN).then_some(value)
}

fn commit_value(value: &str) -> Option<String> {
    let value = single_line(value)?;
    (value == UNKNOWN || (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())))
        .then_some(value)
}

fn git_status_dirty(root: &Path) -> Option<bool> {
    let output = git_output(root, &["status", "--porcelain=v1"])?;
    output
        .status
        .success()
        .then(|| output.stdout.iter().any(|byte| !byte.is_ascii_whitespace()))
}

fn git_stdout(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = git_output(root, arguments)?;
    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
        .and_then(|value| single_line(&value))
}

fn git_output(root: &Path, arguments: &[&str]) -> Option<Output> {
    Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .ok()
}

fn single_line(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && !value.contains('\n') && !value.contains('\r')).then(|| value.to_owned())
}

fn emit(name: &str, value: &str) {
    println!("cargo:rustc-env={name}={value}");
}
