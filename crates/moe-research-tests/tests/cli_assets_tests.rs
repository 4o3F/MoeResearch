use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::write::GzEncoder;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ASSET: &str = "research-skills";
const SKILL_PATH: &str = "skills/deep-research.md";
const PM_SKILL_PATH: &str = "skills/pm-deep-research.md";
const ACADEMIC_SKILL_PATH: &str = "skills/academic-deep-research.md";
const TECHNICAL_SKILL_PATH: &str = "skills/technical-evaluation.md";
const GENERIC_LAYER1_TASK: &str = "prompts/layer1/task-decomposition.md";
const GENERIC_LAYER1_FINAL: &str = "prompts/layer1/final-report.md";
const GENERIC_LAYER2_ASPECT: &str = "prompts/layer2/aspect-agent.md";
const GENERIC_LAYER2_SEARCH_PLANNER: &str = "prompts/layer2/search-planner.md";
const GENERIC_LAYER2_EVIDENCE_EXTRACTOR: &str = "prompts/layer2/evidence-extractor.md";
const COMMON_PATH: &str = "prompts/layer1/common/evidence-postprocess.md";
const MODEL_SEARCH_CONTRACT_PATH: &str = "prompts/layer1/common/model-search-tool-contract.md";
const PM_LAYER1_PATH: &str = "prompts/layer1/pm-deep-research/task-decomposition.md";
const PM_LAYER2_PATH: &str = "prompts/layer2/pm-deep-research/persona-strategist.md";
const ACADEMIC_LAYER1_PATH: &str = "prompts/layer1/academic-deep-research/task-decomposition.md";
const ACADEMIC_LAYER2_PATH: &str =
    "prompts/layer2/academic-deep-research/persona-literature-reviewer.md";
const TECHNICAL_LAYER1_PATH: &str = "prompts/layer1/technical-evaluation/task-decomposition.md";
const TECHNICAL_LAYER2_PATH: &str =
    "prompts/layer2/technical-evaluation/persona-architecture-analyst.md";

const EXPECTED_FILES: &[&str] = &[
    SKILL_PATH,
    PM_SKILL_PATH,
    ACADEMIC_SKILL_PATH,
    TECHNICAL_SKILL_PATH,
    GENERIC_LAYER1_TASK,
    GENERIC_LAYER1_FINAL,
    GENERIC_LAYER2_ASPECT,
    GENERIC_LAYER2_SEARCH_PLANNER,
    GENERIC_LAYER2_EVIDENCE_EXTRACTOR,
    COMMON_PATH,
    MODEL_SEARCH_CONTRACT_PATH,
    PM_LAYER1_PATH,
    PM_LAYER2_PATH,
    ACADEMIC_LAYER1_PATH,
    ACADEMIC_LAYER2_PATH,
    TECHNICAL_LAYER1_PATH,
    TECHNICAL_LAYER2_PATH,
];

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "moe-research-assets-test-{}-{nanos}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Clone)]
struct FixtureFile {
    path: String,
    bytes: Vec<u8>,
}

struct AssetFixture {
    _dir: TestDir,
    base_url: String,
}

impl AssetFixture {
    fn new(
        manifest_override: impl FnOnce(&mut Value),
        extra_archive_file: Option<FixtureFile>,
    ) -> Self {
        let dir = TestDir::new("release");
        let version = env!("CARGO_PKG_VERSION");
        let archive_name = format!("{ASSET}-assets-v{version}.tar.gz");
        let manifest_name = format!("{ASSET}-assets-v{version}.manifest.json");
        let files = fixture_files();
        let archive_bytes = archive_bytes(&files, extra_archive_file.as_ref());
        let mut manifest = json!({
            "schema_version": 1,
            "asset": ASSET,
            "version": version,
            "archive": archive_name,
            "sha256": sha256_hex(&archive_bytes),
            "source_commit": "test",
            "files": files.iter().map(|file| json!({
                "path": file.path,
                "sha256": sha256_hex(&file.bytes),
                "size": file.bytes.len(),
            })).collect::<Vec<_>>(),
        });
        manifest_override(&mut manifest);

        fs::write(dir.path().join(archive_name), archive_bytes).expect("write archive");
        fs::write(
            dir.path().join(manifest_name),
            serde_json::to_vec(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");

        let base_url = url::Url::from_directory_path(dir.path())
            .expect("file base url")
            .to_string();
        Self {
            _dir: dir,
            base_url,
        }
    }
}

#[test]
fn help_exposes_assets_install_options() {
    let output = moeresearch_command()
        .args(["assets", "install", "--help"])
        .output()
        .expect("run assets install help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("research-skills"));
    assert!(stdout.contains("--client"));
    assert!(stdout.contains("--scope"));
    assert!(stdout.contains("--target"));
    assert!(stdout.contains("--layout"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--config"));
}

#[test]
fn package_script_manifest_covers_expanded_research_roots() {
    let output = TestDir::new("package-output");
    let version = "0.0.0-test";

    let command_output = Command::new("node")
        .current_dir(workspace())
        .arg("scripts/package-research-skills-assets.mjs")
        .args([
            "--version",
            version,
            "--output-dir",
            output.path().to_str().expect("utf8 output dir"),
        ])
        .output()
        .expect("run package script");

    assert!(
        command_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&command_output.stderr)
    );

    let archive_path = output
        .path()
        .join(format!("{ASSET}-assets-v{version}.tar.gz"));
    let manifest_path = output
        .path()
        .join(format!("{ASSET}-assets-v{version}.manifest.json"));
    assert!(archive_path.is_file());
    assert!(manifest_path.is_file());

    let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).expect("read manifest"))
        .expect("parse manifest");
    assert_eq!(manifest["asset"], ASSET);
    assert_eq!(
        manifest["sha256"],
        sha256_hex(&fs::read(archive_path).expect("read archive"))
    );

    let paths = manifest["files"]
        .as_array()
        .expect("manifest files")
        .iter()
        .map(|file| file["path"].as_str().expect("file path").to_owned())
        .collect::<BTreeSet<_>>();
    for expected in EXPECTED_FILES {
        assert!(paths.contains(*expected), "manifest missing {expected}");
    }
    for path in &paths {
        assert!(
            is_research_asset_path(path),
            "unexpected packaged path {path}"
        );
    }
}

#[test]
fn assets_install_repo_layout_preserves_skills_and_prompts() {
    let fixture = AssetFixture::new(|_| {}, None);
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        fs::read_to_string(target.path().join(SKILL_PATH)).expect("read skill"),
        skill_content()
    );
    for expected in EXPECTED_FILES {
        assert!(target.path().join(expected).is_file(), "missing {expected}");
    }
}

#[test]
fn assets_install_default_claude_code_layout_rewrites_skill_prompt_paths() {
    let fixture = AssetFixture::new(|_| {}, None);
    let home = TestDir::new("home");

    let output = moeresearch_command()
        .env("HOME", home.path())
        .args([
            "assets",
            "install",
            "research-skills",
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--base-url",
            &fixture.base_url,
        ])
        .output()
        .expect("run assets install");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());

    let skill_root = home.path().join(".claude/skills/deep-research");
    let installed_skill = fs::read_to_string(skill_root.join("SKILL.md")).expect("read skill");
    assert!(installed_skill.contains("./prompts/layer1/pm-deep-research/task-decomposition.md"));
    assert!(
        installed_skill.contains("./prompts/layer1/academic-deep-research/task-decomposition.md")
    );
    assert!(!installed_skill.contains("../prompts/"));
    assert!(!skill_root.join(SKILL_PATH).exists());
    assert!(!skill_root.join(ACADEMIC_SKILL_PATH).exists());
    assert!(!skill_root.join(TECHNICAL_SKILL_PATH).exists());
    assert!(
        skill_root
            .join("prompts/layer1/common/evidence-postprocess.md")
            .is_file()
    );
    let contract =
        fs::read_to_string(skill_root.join("prompts/layer1/common/model-search-tool-contract.md"))
            .expect("read installed model search contract");
    assert!(
        contract.contains("Run Binding")
            && contract.contains("moe.run_binding.v1")
            && contract.contains("allowed_source_focus")
            && contract.contains("selected_evidence_rule"),
        "installed common contract must retain Run Binding projection and closure guidance"
    );
    assert!(
        skill_root
            .join("prompts/layer1/academic-deep-research/task-decomposition.md")
            .is_file()
    );
    assert!(
        skill_root
            .join("prompts/layer1/technical-evaluation/task-decomposition.md")
            .is_file()
    );
    assert!(
        skill_root
            .join("prompts/layer2/pm-deep-research/persona-strategist.md")
            .is_file()
    );
    assert!(
        skill_root
            .join("prompts/layer1/task-decomposition.md")
            .is_file()
    );
    assert!(skill_root.join("prompts/layer1/final-report.md").is_file());
    assert!(skill_root.join("prompts/layer2/aspect-agent.md").is_file());
    assert!(
        skill_root
            .join("prompts/layer2/search-planner.md")
            .is_file()
    );
    assert!(
        skill_root
            .join("prompts/layer2/evidence-extractor.md")
            .is_file()
    );
}

#[test]
fn packaging_allowlists_are_isomorphic() {
    let rust_source =
        fs::read_to_string(workspace().join("crates/moe-research-cli/src/commands/assets.rs"))
            .expect("read assets.rs");
    let node_source =
        fs::read_to_string(workspace().join("scripts/package-research-skills-assets.mjs"))
            .expect("read package script");

    let rust_files = extract_string_literals_in_const_block(&rust_source, "ALLOWED_ASSET_FILES");
    let rust_prefixes =
        extract_string_literals_in_const_block(&rust_source, "ALLOWED_ASSET_PREFIXES");
    let node_files = extract_string_literals_in_const_block(&node_source, "ALLOWED_FILES");
    let node_prefixes = extract_string_literals_in_const_block(&node_source, "ALLOWED_PREFIXES");

    assert_eq!(
        rust_files, node_files,
        "ALLOWED_ASSET_FILES and ALLOWED_FILES must be isomorphic"
    );
    assert_eq!(
        rust_prefixes, node_prefixes,
        "ALLOWED_ASSET_PREFIXES and ALLOWED_PREFIXES must be isomorphic"
    );

    for expected in [
        GENERIC_LAYER1_TASK,
        GENERIC_LAYER1_FINAL,
        GENERIC_LAYER2_ASPECT,
        GENERIC_LAYER2_SEARCH_PLANNER,
        GENERIC_LAYER2_EVIDENCE_EXTRACTOR,
    ] {
        assert!(
            rust_files.contains(expected),
            "shared allowlist missing Generic path {expected}"
        );
    }
}

#[test]
fn assets_install_project_scope_uses_current_project_directory() {
    let fixture = AssetFixture::new(|_| {}, None);
    let project = TestDir::new("project");

    let output = moeresearch_command()
        .current_dir(project.path())
        .args([
            "assets",
            "install",
            "research-skills",
            "--scope",
            "project",
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--base-url",
            &fixture.base_url,
        ])
        .output()
        .expect("run assets install");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(
        project
            .path()
            .join(".claude/skills/deep-research/SKILL.md")
            .is_file()
    );
}

#[test]
fn assets_install_dry_run_does_not_write_target_files() {
    let fixture = AssetFixture::new(|_| {}, None);
    let target = TestDir::new("target");

    let output = moeresearch_command()
        .args([
            "assets",
            "install",
            "research-skills",
            "--target",
            target.path().to_str().expect("utf8 target"),
            "--layout",
            "repo",
            "--dry-run",
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--base-url",
            &fixture.base_url,
        ])
        .output()
        .expect("run assets install");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr: {stderr}");
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
    assert!(stderr.contains("would install MoeResearch research assets"));
}

#[test]
fn assets_install_rejects_archive_checksum_mismatch() {
    let fixture = AssetFixture::new(
        |manifest| {
            manifest["sha256"] = Value::String("0".repeat(64));
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_per_file_checksum_mismatch() {
    let fixture = AssetFixture::new(
        |manifest| {
            manifest["files"][0]["sha256"] = Value::String("0".repeat(64));
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_archive_traversal_entry() {
    let fixture = AssetFixture::new(
        |_| {},
        Some(FixtureFile {
            path: "../evil.txt".to_owned(),
            bytes: b"evil".to_vec(),
        }),
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_archive_unexpected_file() {
    let fixture = AssetFixture::new(
        |_| {},
        Some(FixtureFile {
            path: "prompts/layer1/pm-deep-research/extra.md".to_owned(),
            bytes: b"extra".to_vec(),
        }),
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_manifest_duplicate_path() {
    let fixture = AssetFixture::new(
        |manifest| {
            let duplicate = manifest["files"][0].clone();
            manifest["files"]
                .as_array_mut()
                .expect("files array")
                .push(duplicate);
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_version_mismatch() {
    let fixture = AssetFixture::new(
        |manifest| {
            manifest["version"] = Value::String("9.9.9".to_owned());
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_manifest_unrelated_path() {
    let fixture = AssetFixture::new(
        |manifest| {
            manifest["files"][0]["path"] = Value::String("docs/not-research.md".to_owned());
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_rejects_manifest_prompt_directory_as_file() {
    let fixture = AssetFixture::new(
        |manifest| {
            manifest["files"][0]["path"] = Value::String("prompts/layer1/common".to_owned());
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_requires_manifest_core_skill_file() {
    let fixture = AssetFixture::new(
        |manifest| {
            manifest["files"]
                .as_array_mut()
                .expect("files array")
                .retain(|file| file["path"] != SKILL_PATH);
        },
        None,
    );
    let target = TestDir::new("target");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!target.path().join(SKILL_PATH).exists());
}

#[test]
fn assets_install_preserves_conflicting_files_without_force() {
    let fixture = AssetFixture::new(|_| {}, None);
    let target = TestDir::new("target");
    let skill_path = target.path().join(SKILL_PATH);
    fs::create_dir_all(skill_path.parent().expect("skill parent")).expect("create parent");
    fs::write(&skill_path, "local edit").expect("write local edit");

    let output = install_repo_layout(&fixture, &target);

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(skill_path).expect("read local edit"),
        "local edit"
    );
}

#[test]
fn assets_install_force_overwrites_manifest_owned_file_only() {
    let fixture = AssetFixture::new(|_| {}, None);
    let target = TestDir::new("target");
    let skill_path = target.path().join(SKILL_PATH);
    let unknown_path = target.path().join("skills/local.md");
    fs::create_dir_all(skill_path.parent().expect("skill parent")).expect("create parent");
    fs::write(&skill_path, "local edit").expect("write local edit");
    fs::write(&unknown_path, "keep me").expect("write unknown");

    let output = moeresearch_command()
        .args([
            "assets",
            "install",
            "research-skills",
            "--target",
            target.path().to_str().expect("utf8 target"),
            "--layout",
            "repo",
            "--force",
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--base-url",
            &fixture.base_url,
        ])
        .output()
        .expect("run assets install");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(skill_path).expect("read installed skill"),
        skill_content()
    );
    assert_eq!(
        fs::read_to_string(unknown_path).expect("read unknown"),
        "keep me"
    );
}

#[test]
fn assets_install_routes_remote_downloads_through_configured_socks5h_proxy() {
    let proxy = SocksAssetServer::new();
    let target = TestDir::new("socks-target");
    let config_dir = TestDir::new("socks-config");
    let config_path = config_dir.path().join("moeresearch.toml");
    write_disabled_config_with_proxy(&config_path, &proxy.proxy_url);

    let output = moeresearch_command()
        .env("NO_PROXY", "assets.invalid")
        .args([
            "assets",
            "install",
            "research-skills",
            "--target",
            target.path().to_str().expect("UTF-8 target path"),
            "--layout",
            "repo",
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--base-url",
            "http://assets.invalid/releases/",
            "--config",
            config_path.to_str().expect("UTF-8 config path"),
        ])
        .output()
        .expect("run assets install");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(target.path().join(SKILL_PATH).is_file());

    let destinations = proxy
        .destinations
        .lock()
        .expect("SOCKS destination lock")
        .clone();
    assert_eq!(
        destinations,
        vec![
            (0x03, "assets.invalid".to_owned()),
            (0x03, "assets.invalid".to_owned()),
        ]
    );
}

fn install_repo_layout(fixture: &AssetFixture, target: &TestDir) -> std::process::Output {
    moeresearch_command()
        .args([
            "assets",
            "install",
            "research-skills",
            "--target",
            target.path().to_str().expect("utf8 target"),
            "--layout",
            "repo",
            "--version",
            env!("CARGO_PKG_VERSION"),
            "--base-url",
            &fixture.base_url,
        ])
        .output()
        .expect("run assets install")
}

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .to_path_buf()
}

fn moeresearch_command() -> Command {
    let mut command = Command::new(env!("CARGO"));
    command
        .current_dir(workspace())
        .args(["run", "--quiet", "--locked", "--manifest-path"])
        .arg(workspace().join("Cargo.toml"))
        .args(["-p", "moe-research-cli", "--"]);
    command
}

fn fixture_files() -> Vec<FixtureFile> {
    EXPECTED_FILES
        .iter()
        .map(|path| FixtureFile {
            path: (*path).to_owned(),
            bytes: fixture_content(path),
        })
        .collect()
}

fn skill_content() -> String {
    concat!(
        "Use ../prompts/layer1/pm-deep-research/task-decomposition.md\n",
        "Use ../prompts/layer1/academic-deep-research/task-decomposition.md\n",
    )
    .to_owned()
}

fn fixture_content(path: &str) -> Vec<u8> {
    match path {
        SKILL_PATH => skill_content().into_bytes(),
        PM_SKILL_PATH => b"pm research reference skill".to_vec(),
        ACADEMIC_SKILL_PATH => b"academic research reference skill".to_vec(),
        TECHNICAL_SKILL_PATH => b"technical evaluation reference skill".to_vec(),
        MODEL_SEARCH_CONTRACT_PATH => {
            include_bytes!("../../../prompts/layer1/common/model-search-tool-contract.md").to_vec()
        }
        _ => format!("fixture prompt for {path}").into_bytes(),
    }
}

fn is_research_asset_path(path: &str) -> bool {
    matches!(
        path,
        SKILL_PATH
            | PM_SKILL_PATH
            | ACADEMIC_SKILL_PATH
            | TECHNICAL_SKILL_PATH
            | GENERIC_LAYER1_TASK
            | GENERIC_LAYER1_FINAL
            | GENERIC_LAYER2_ASPECT
            | GENERIC_LAYER2_SEARCH_PLANNER
            | GENERIC_LAYER2_EVIDENCE_EXTRACTOR
    ) || path.starts_with("prompts/layer1/common/")
        || path.starts_with("prompts/layer1/pm-deep-research/")
        || path.starts_with("prompts/layer2/pm-deep-research/")
        || path.starts_with("prompts/layer1/academic-deep-research/")
        || path.starts_with("prompts/layer2/academic-deep-research/")
        || path.starts_with("prompts/layer1/technical-evaluation/")
        || path.starts_with("prompts/layer2/technical-evaluation/")
}

fn extract_string_literals_in_const_block(source: &str, const_name: &str) -> BTreeSet<String> {
    // Locate `NAME =` / `NAME:` declaration, then scan from the initializer `[`
    // so Rust type signatures like `&[&str]` do not terminate early.
    let decl_patterns = [format!("const {const_name}"), format!("{const_name} =")];
    let mut search_from = 0;
    let decl_start = loop {
        let relative = source[search_from..]
            .find(const_name)
            .unwrap_or_else(|| panic!("const block not found: {const_name}"));
        let absolute = search_from + relative;
        let line_start = source[..absolute].rfind('\n').map_or(0, |idx| idx + 1);
        let line_end = source[absolute..]
            .find('\n')
            .map_or(source.len(), |idx| absolute + idx);
        let line = &source[line_start..line_end];
        if decl_patterns
            .iter()
            .any(|pattern| line.contains(pattern.as_str()))
            && (line.contains('=') || line.contains(':'))
        {
            break absolute;
        }
        search_from = absolute + const_name.len();
    };

    let after_decl = &source[decl_start..];
    let bracket_rel = after_decl
        .find('[')
        .unwrap_or_else(|| panic!("const initializer not found for {const_name}"));
    // Prefer the last `[` on the declaration/initializer header line so
    // `const NAME: &[&str] = &[` starts at the array literal, not the type.
    let header_line_end = after_decl[bracket_rel..]
        .find('\n')
        .map_or(after_decl.len(), |idx| bracket_rel + idx);
    let header = &after_decl[..header_line_end];
    let array_start = header.rfind('[').expect("array start");
    let body = &after_decl[array_start + 1..];
    let mut values = BTreeSet::new();
    let mut depth = 1_i32;
    let mut index = 0;
    let bytes = body.as_bytes();
    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            b'"' | b'\'' => {
                let quote = byte;
                index += 1;
                let start = index;
                while index < bytes.len() && bytes[index] != quote {
                    index += 1;
                }
                if index >= bytes.len() {
                    break;
                }
                let value = &body[start..index];
                if value.contains('/') || value.ends_with(".md") {
                    values.insert(value.to_owned());
                }
            }
            _ => {}
        }
        index += 1;
    }
    if depth != 0 {
        panic!("unterminated const block for {const_name}");
    }
    values
}

fn archive_bytes(files: &[FixtureFile], extra_file: Option<&FixtureFile>) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    for file in files.iter().chain(extra_file) {
        append_tar_file(&mut tar_bytes, file);
    }
    tar_bytes.extend_from_slice(&[0; 1024]);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).expect("write gzip");
    encoder.finish().expect("finish gzip")
}

fn append_tar_file(tar_bytes: &mut Vec<u8>, file: &FixtureFile) {
    let mut header = [0_u8; 512];
    write_tar_string(&mut header, 0, 100, &file.path);
    write_tar_octal(&mut header, 100, 8, 0o644);
    write_tar_octal(&mut header, 108, 8, 0);
    write_tar_octal(&mut header, 116, 8, 0);
    write_tar_octal(
        &mut header,
        124,
        12,
        file.bytes.len().try_into().expect("file size fits"),
    );
    write_tar_octal(&mut header, 136, 12, 0);
    header[148..156].fill(b' ');
    header[156] = b'0';
    write_tar_string(&mut header, 257, 6, "ustar");
    write_tar_string(&mut header, 263, 2, "00");

    let checksum = header.iter().map(|byte| u64::from(*byte)).sum();
    write_tar_octal(&mut header, 148, 8, checksum);

    tar_bytes.extend_from_slice(&header);
    tar_bytes.extend_from_slice(&file.bytes);
    tar_bytes.extend(std::iter::repeat_n(0, tar_padding(file.bytes.len())));
}

fn write_tar_string(header: &mut [u8; 512], offset: usize, len: usize, value: &str) {
    let bytes = value.as_bytes();
    assert!(bytes.len() <= len, "tar value is too long");
    header[offset..offset + bytes.len()].copy_from_slice(bytes);
}

fn write_tar_octal(header: &mut [u8; 512], offset: usize, len: usize, value: u64) {
    let text = format!("{value:0width$o}", width = len - 1);
    assert!(text.len() < len, "tar octal value is too long");
    header[offset..offset + text.len()].copy_from_slice(text.as_bytes());
}

fn tar_padding(size: usize) -> usize {
    let remainder = size % 512;
    if remainder == 0 { 0 } else { 512 - remainder }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hash.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

struct SocksAssetServer {
    proxy_url: String,
    destinations: Arc<Mutex<Vec<(u8, String)>>>,
    _thread: thread::JoinHandle<()>,
}

impl SocksAssetServer {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind SOCKS asset proxy");
        let address = listener.local_addr().expect("SOCKS asset proxy address");
        let version = env!("CARGO_PKG_VERSION");
        let archive_name = format!("{ASSET}-assets-v{version}.tar.gz");
        let files = fixture_files();
        let archive = archive_bytes(&files, None);
        let manifest = serde_json::to_vec(&json!({
            "schema_version": 1,
            "asset": ASSET,
            "version": version,
            "archive": archive_name,
            "sha256": sha256_hex(&archive),
            "source_commit": "test",
            "files": files.iter().map(|file| json!({
                "path": file.path,
                "sha256": sha256_hex(&file.bytes),
                "size": file.bytes.len(),
            })).collect::<Vec<_>>(),
        }))
        .expect("serialize asset manifest");
        let destinations = Arc::new(Mutex::new(Vec::new()));
        let destinations_for_thread = Arc::clone(&destinations);
        let thread = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept SOCKS asset request");
                let destination = read_blocking_socks5_connect(&mut stream);
                destinations_for_thread
                    .lock()
                    .expect("SOCKS destination lock")
                    .push(destination);
                let request_line = read_blocking_http_request(&mut stream);
                let body = if request_line.contains(".manifest.json") {
                    &manifest
                } else {
                    &archive
                };
                write_blocking_http_response(&mut stream, body);
            }
        });

        Self {
            proxy_url: format!("socks5h://{address}"),
            destinations,
            _thread: thread,
        }
    }
}

fn read_blocking_socks5_connect(stream: &mut TcpStream) -> (u8, String) {
    let mut greeting = [0_u8; 2];
    stream
        .read_exact(&mut greeting)
        .expect("read SOCKS greeting");
    assert_eq!(greeting[0], 0x05);
    let mut methods = vec![0_u8; usize::from(greeting[1])];
    stream.read_exact(&mut methods).expect("read SOCKS methods");
    assert!(methods.contains(&0x00));
    stream
        .write_all(&[0x05, 0x00])
        .expect("accept SOCKS no-auth method");

    let mut request = [0_u8; 4];
    stream
        .read_exact(&mut request)
        .expect("read SOCKS CONNECT header");
    assert_eq!(request[..3], [0x05, 0x01, 0x00]);
    let address_type = request[3];
    let destination = match address_type {
        0x03 => {
            let mut length = [0_u8; 1];
            stream
                .read_exact(&mut length)
                .expect("read SOCKS domain length");
            let mut domain = vec![0_u8; usize::from(length[0])];
            stream.read_exact(&mut domain).expect("read SOCKS domain");
            String::from_utf8(domain).expect("SOCKS domain is UTF-8")
        }
        0x01 => {
            let mut octets = [0_u8; 4];
            stream
                .read_exact(&mut octets)
                .expect("read SOCKS IPv4 target");
            std::net::Ipv4Addr::from(octets).to_string()
        }
        other => panic!("unexpected SOCKS address type {other}"),
    };
    let mut port = [0_u8; 2];
    stream
        .read_exact(&mut port)
        .expect("read SOCKS target port");
    stream
        .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .expect("accept SOCKS CONNECT");

    (address_type, destination)
}

fn read_blocking_http_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut buffer).expect("read asset HTTP request");
        assert_ne!(read, 0, "asset request ended before headers");
        bytes.extend_from_slice(&buffer[..read]);
    }

    String::from_utf8(bytes)
        .expect("asset request headers are UTF-8")
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn write_blocking_http_response(stream: &mut TcpStream, body: &[u8]) {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .expect("write asset response headers");
    stream.write_all(body).expect("write asset response body");
}

fn write_disabled_config_with_proxy(path: &Path, proxy_url: &str) {
    fs::write(
        path,
        format!(
            r#"[logging]
format = "json"

[network]
inactivity_timeout_ms = 30000
max_retries = 0
retry_backoff_ms = 1
user_agent = "moeresearch-test/0.0.0"
proxy_url = "{proxy_url}"

[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
inactivity_timeout_ms = 30000

[search.providers.tavily]
enabled = false
base_url = "https://api.tavily.com"
api_key_env = "TAVILY_API_KEY"
inactivity_timeout_ms = 30000

[search.providers.grok]
enabled = false
base_url = "https://api.x.ai/v1"
api_key_env = "XAI_API_KEY"
inactivity_timeout_ms = 30000
model = "grok-4.3"

[model.providers.openai]
enabled = false
base_url = "https://api.openai.com/v1"
api_key_env = "OPENAI_API_KEY"
inactivity_timeout_ms = 30000
model = "gpt-5.5"

[limits.research]
max_agents = -1
max_concurrent_agents = -1
max_total_model_calls = -1
max_total_search_calls = -1
total_timeout_ms = -1
max_tokens = -1

[limits.per_agent]
max_turns = -1
max_tool_calls = -1
max_search_calls = -1
timeout_ms = -1
"#
        ),
    )
    .expect("write proxy test config");
}
