#![allow(clippy::must_use_candidate, clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use flate2::read::GzDecoder;
use moe_research_error::{Error, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::info;
use url::Url;

const ASSET_RESEARCH_SKILLS: &str = "research-skills";
const CLAUDE_SKILL_NAME: &str = "deep-research";
const DEFAULT_REPOSITORY: &str = "https://github.com/4o3F/MoeResearch";
const MANIFEST_SCHEMA_VERSION: u32 = 1;
const SKILL_SOURCE_PATH: &str = "skills/deep-research.md";

const ALLOWED_ASSET_FILES: &[&str] = &[
    "skills/deep-research.md",
    "skills/pm-deep-research.md",
    "skills/academic-deep-research.md",
    "skills/technical-evaluation.md",
];

const ALLOWED_ASSET_PREFIXES: &[&str] = &[
    "prompts/layer1/common/",
    "prompts/layer1/pm-deep-research/",
    "prompts/layer2/pm-deep-research/",
    "prompts/layer1/academic-deep-research/",
    "prompts/layer2/academic-deep-research/",
    "prompts/layer1/technical-evaluation/",
    "prompts/layer2/technical-evaluation/",
];

const ALLOWED_ASSET_DIRS: &[&str] = &[
    "skills",
    "prompts",
    "prompts/layer1",
    "prompts/layer2",
    "prompts/layer1/common",
    "prompts/layer1/pm-deep-research",
    "prompts/layer2/pm-deep-research",
    "prompts/layer1/academic-deep-research",
    "prompts/layer2/academic-deep-research",
    "prompts/layer1/technical-evaluation",
    "prompts/layer2/technical-evaluation",
];

#[derive(Debug, Args)]
pub struct AssetsArgs {
    #[command(subcommand)]
    pub command: AssetsCommand,
}

#[derive(Debug, Subcommand)]
pub enum AssetsCommand {
    /// Install released MoeResearch assets.
    Install(InstallArgs),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum AssetName {
    ResearchSkills,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum ClientName {
    ClaudeCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum InstallLayout {
    ClaudeCode,
    Repo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum InstallScope {
    User,
    Project,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Asset bundle to install.
    pub asset: AssetName,
    /// Client-specific install target. Defaults to claude-code when --target is omitted.
    #[arg(long)]
    pub client: Option<ClientName>,
    /// Client install scope. Only valid for --client claude-code.
    #[arg(long)]
    pub scope: Option<InstallScope>,
    /// Generic asset root. Use with --layout repo for sibling skills/ and prompts/ dirs.
    #[arg(long)]
    pub target: Option<PathBuf>,
    /// Install layout. Defaults to claude-code without --target and repo with --target.
    #[arg(long)]
    pub layout: Option<InstallLayout>,
    /// Overwrite differing manifest-owned destination files.
    #[arg(long)]
    pub force: bool,
    /// Resolve and validate install inputs without writing target files.
    #[arg(long)]
    pub dry_run: bool,
    /// Asset version to install. Defaults to this binary package version.
    #[arg(long)]
    pub version: Option<String>,
    /// Override release download base URL for tests or mirrors.
    #[arg(long, hide = true)]
    pub base_url: Option<Url>,
}

#[derive(Debug, Deserialize)]
struct AssetManifest {
    schema_version: u32,
    asset: String,
    version: String,
    archive: String,
    sha256: String,
    files: Vec<ManifestFile>,
}

#[derive(Debug, Deserialize)]
struct ManifestFile {
    path: String,
    sha256: String,
    size: u64,
}

#[derive(Debug)]
struct InstallTarget {
    root: PathBuf,
    layout: InstallLayout,
}

#[derive(Debug)]
struct PlannedFile {
    source_path: PathBuf,
    destination_path: PathBuf,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct Conflict {
    path: PathBuf,
    replaceable: bool,
}

pub async fn run(args: AssetsArgs) -> Result<()> {
    match args.command {
        AssetsCommand::Install(args) => install(args).await,
    }
}

async fn install(args: InstallArgs) -> Result<()> {
    let asset = asset_slug(args.asset);
    let version = args
        .version
        .as_deref()
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_owned();
    let target = resolve_install_target(&args)?;
    let base_url = normalize_base_url(
        args.base_url
            .unwrap_or_else(|| release_base_url(DEFAULT_REPOSITORY, &version)),
    );
    let manifest_name = manifest_filename(asset, &version);
    let archive_name = archive_filename(asset, &version);

    let manifest_url = join_url(&base_url, &manifest_name)?;
    let archive_url = join_url(&base_url, &archive_name)?;
    let manifest_bytes = download_bytes(&manifest_url).await?;
    let manifest = parse_manifest(&manifest_bytes, asset, &version, &archive_name)?;

    let archive_bytes = download_bytes(&archive_url).await?;
    verify_bytes_sha256(&archive_bytes, &manifest.sha256, "archive")?;

    let stage = tempfile::Builder::new()
        .prefix("moeresearch-assets-")
        .tempdir()
        .map_err(io_error("create asset staging directory"))?;
    extract_archive(&archive_bytes, stage.path(), &manifest)?;
    let planned_files = plan_install(stage.path(), &target, &manifest)?;
    let conflicts = find_conflicts(&planned_files)?;

    if args.dry_run {
        info!(
            asset,
            version,
            layout = ?target.layout,
            target = %target.root.display(),
            file_count = planned_files.len(),
            conflict_count = conflicts.len(),
            "would install MoeResearch research assets"
        );
        return Ok(());
    }

    reject_blocking_conflicts(&conflicts, args.force)?;
    write_planned_files(&planned_files)?;
    info!(
        asset,
        version,
        layout = ?target.layout,
        target = %target.root.display(),
        file_count = planned_files.len(),
        "installed MoeResearch research assets"
    );
    Ok(())
}

fn resolve_install_target(args: &InstallArgs) -> Result<InstallTarget> {
    match (&args.target, args.client, args.scope, args.layout) {
        (Some(_), Some(_), _, _) => Err(Error::InvalidInput {
            message: "--target cannot be combined with --client".to_owned(),
        }),
        (Some(_), _, Some(_), _) => Err(Error::InvalidInput {
            message: "--target cannot be combined with --scope".to_owned(),
        }),
        (Some(target), None, None, layout) => {
            let layout = layout.unwrap_or(InstallLayout::Repo);
            if layout != InstallLayout::Repo {
                return Err(Error::InvalidInput {
                    message: "--target currently supports --layout repo".to_owned(),
                });
            }
            Ok(InstallTarget {
                root: target.clone(),
                layout,
            })
        }
        (None, client, scope, layout) => {
            let client = client.unwrap_or(ClientName::ClaudeCode);
            if client != ClientName::ClaudeCode {
                return Err(Error::InvalidInput {
                    message: "unsupported assets client".to_owned(),
                });
            }
            let layout = layout.unwrap_or(InstallLayout::ClaudeCode);
            if layout != InstallLayout::ClaudeCode {
                return Err(Error::InvalidInput {
                    message: "--layout repo requires --target".to_owned(),
                });
            }
            Ok(InstallTarget {
                root: claude_code_skill_root(scope.unwrap_or(InstallScope::User))?,
                layout,
            })
        }
    }
}

fn claude_code_skill_root(scope: InstallScope) -> Result<PathBuf> {
    match scope {
        InstallScope::Project => Ok(std::env::current_dir()
            .map_err(io_error("resolve current directory"))?
            .join(".claude")
            .join("skills")
            .join(CLAUDE_SKILL_NAME)),
        InstallScope::User => Ok(home_dir()?
            .join(".claude")
            .join("skills")
            .join(CLAUDE_SKILL_NAME)),
    }
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| Error::InvalidInput {
            message: "cannot resolve home directory for Claude Code user skill install".to_owned(),
        })
}

fn asset_slug(asset: AssetName) -> &'static str {
    match asset {
        AssetName::ResearchSkills => ASSET_RESEARCH_SKILLS,
    }
}

fn release_base_url(repository: &str, version: &str) -> Url {
    Url::parse(&format!("{repository}/releases/download/v{version}/"))
        .expect("static GitHub release URL is valid")
}

fn normalize_base_url(mut base_url: Url) -> Url {
    if !base_url.path().ends_with('/') {
        let path = format!("{}/", base_url.path());
        base_url.set_path(&path);
    }
    base_url
}

fn manifest_filename(asset: &str, version: &str) -> String {
    format!("{asset}-assets-v{version}.manifest.json")
}

fn archive_filename(asset: &str, version: &str) -> String {
    format!("{asset}-assets-v{version}.tar.gz")
}

fn join_url(base_url: &Url, filename: &str) -> Result<Url> {
    base_url
        .join(filename)
        .map_err(|source| Error::InvalidInput {
            message: format!("invalid asset URL: {source}"),
        })
}

async fn download_bytes(url: &Url) -> Result<Vec<u8>> {
    if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|()| Error::InvalidInput {
            message: format!("invalid file asset URL: {url}"),
        })?;
        return fs::read(path).map_err(io_error("read local asset file"));
    }

    let response = reqwest::Client::new()
        .get(url.clone())
        .send()
        .await
        .map_err(|source| network_error(&source))?;
    let status = response.status();
    if !status.is_success() {
        return Err(Error::NetworkFailed {
            message: format!("asset download returned HTTP {}", status.as_u16()),
        });
    }
    response
        .bytes()
        .await
        .map(Vec::from)
        .map_err(|source| network_error(&source))
}

fn parse_manifest(
    bytes: &[u8],
    asset: &str,
    version: &str,
    archive_name: &str,
) -> Result<AssetManifest> {
    let manifest: AssetManifest =
        serde_json::from_slice(bytes).map_err(|source| Error::Json { source })?;

    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(Error::InvalidInput {
            message: "unsupported asset manifest schema_version".to_owned(),
        });
    }
    if manifest.asset != asset || manifest.version != version || manifest.archive != archive_name {
        return Err(Error::InvalidInput {
            message: "asset manifest does not match requested asset/version".to_owned(),
        });
    }
    if manifest.files.is_empty() {
        return Err(Error::InvalidInput {
            message: "asset manifest must list files".to_owned(),
        });
    }
    if !manifest
        .files
        .iter()
        .any(|file| file.path == SKILL_SOURCE_PATH)
    {
        return Err(Error::InvalidInput {
            message: format!("asset manifest must include {SKILL_SOURCE_PATH}"),
        });
    }
    validate_sha256_hex(&manifest.sha256, "archive sha256")?;

    let mut seen = BTreeSet::new();
    for file in &manifest.files {
        validate_manifest_file(file)?;
        if !seen.insert(file.path.clone()) {
            return Err(Error::InvalidInput {
                message: format!("asset manifest contains duplicate path {}", file.path),
            });
        }
    }

    Ok(manifest)
}

fn validate_manifest_file(file: &ManifestFile) -> Result<()> {
    let path = Path::new(&file.path);
    validate_safe_relative_path(path)?;
    validate_owned_asset_file_path(path)?;
    validate_sha256_hex(&file.sha256, &format!("{} sha256", file.path))?;
    Ok(())
}

fn validate_safe_relative_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() || path.to_string_lossy().contains('\\') {
        return Err(Error::InvalidInput {
            message: format!("unsafe asset path {}", path.display()),
        });
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::Prefix(_)
            | Component::RootDir => {
                return Err(Error::InvalidInput {
                    message: format!("unsafe asset path {}", path.display()),
                });
            }
        }
    }
    Ok(())
}

fn validate_owned_asset_file_path(path: &Path) -> Result<()> {
    let allowed_file = ALLOWED_ASSET_FILES
        .iter()
        .any(|allowed| path == Path::new(allowed));
    let allowed_prompt = ALLOWED_ASSET_PREFIXES
        .iter()
        .any(|prefix| path_has_child_under(path, Path::new(prefix)));

    if allowed_file || allowed_prompt {
        return Ok(());
    }

    Err(Error::InvalidInput {
        message: format!("unexpected asset path {}", path.display()),
    })
}

fn validate_allowed_asset_dir_path(path: &Path) -> Result<()> {
    if ALLOWED_ASSET_DIRS
        .iter()
        .any(|allowed| path == Path::new(allowed))
    {
        return Ok(());
    }

    Err(Error::InvalidInput {
        message: format!("unexpected asset directory {}", path.display()),
    })
}

fn path_has_child_under(path: &Path, directory: &Path) -> bool {
    path.strip_prefix(directory)
        .is_ok_and(|remainder| remainder.components().next().is_some())
}

fn validate_sha256_hex(value: &str, label: &str) -> Result<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(Error::InvalidInput {
        message: format!("{label} must be a lowercase hex sha256"),
    })
}

fn extract_archive(bytes: &[u8], stage: &Path, manifest: &AssetManifest) -> Result<()> {
    let expected_files = manifest
        .files
        .iter()
        .map(|file| (PathBuf::from(&file.path), file))
        .collect::<BTreeMap<_, _>>();
    let decoder = GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    let mut extracted_files = BTreeSet::new();

    for entry in archive.entries().map_err(io_error("read asset archive"))? {
        let mut entry = entry.map_err(io_error("read asset archive entry"))?;
        let path = entry
            .path()
            .map_err(io_error("read asset archive path"))?
            .into_owned();
        validate_safe_relative_path(&path)?;

        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            validate_allowed_asset_dir_path(&path)?;
            fs::create_dir_all(stage.join(&path)).map_err(io_error("create staged directory"))?;
            continue;
        }
        if !entry_type.is_file() {
            return Err(Error::InvalidInput {
                message: format!(
                    "asset archive contains unsupported entry type at {}",
                    path.display()
                ),
            });
        }
        validate_owned_asset_file_path(&path)?;

        let Some(expected) = expected_files.get(&path) else {
            return Err(Error::InvalidInput {
                message: format!("asset archive contains unexpected file {}", path.display()),
            });
        };
        if !extracted_files.insert(path.clone()) {
            return Err(Error::InvalidInput {
                message: format!("asset archive contains duplicate file {}", path.display()),
            });
        }

        let destination = stage.join(&path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(io_error("create staged directory"))?;
        }
        entry
            .unpack(&destination)
            .map_err(io_error("extract asset archive entry"))?;
        verify_staged_file(&destination, expected)?;
    }

    for file in &manifest.files {
        if !extracted_files.contains(Path::new(&file.path)) {
            return Err(Error::InvalidInput {
                message: format!("asset archive is missing {}", file.path),
            });
        }
    }
    Ok(())
}

fn verify_staged_file(path: &Path, expected: &ManifestFile) -> Result<()> {
    let metadata = fs::metadata(path).map_err(io_error("stat staged asset file"))?;
    if metadata.len() != expected.size {
        return Err(Error::InvalidInput {
            message: format!("asset file size mismatch for {}", expected.path),
        });
    }
    let actual = file_sha256(path)?;
    if actual != expected.sha256 {
        return Err(Error::InvalidInput {
            message: format!("asset file checksum mismatch for {}", expected.path),
        });
    }
    Ok(())
}

fn plan_install(
    stage: &Path,
    target: &InstallTarget,
    manifest: &AssetManifest,
) -> Result<Vec<PlannedFile>> {
    let mut planned = Vec::with_capacity(manifest.files.len());
    for file in &manifest.files {
        let source_path = PathBuf::from(&file.path);
        let staged_path = stage.join(&source_path);
        let bytes = fs::read(&staged_path).map_err(io_error("read staged asset file"))?;
        if let Some((destination_path, bytes)) = map_install_file(&source_path, bytes, target)? {
            planned.push(PlannedFile {
                source_path,
                destination_path,
                bytes,
            });
        }
    }
    Ok(planned)
}

fn map_install_file(
    source_path: &Path,
    bytes: Vec<u8>,
    target: &InstallTarget,
) -> Result<Option<(PathBuf, Vec<u8>)>> {
    match target.layout {
        InstallLayout::Repo => Ok(Some((target.root.join(source_path), bytes))),
        InstallLayout::ClaudeCode => {
            if source_path == Path::new(SKILL_SOURCE_PATH) {
                let content = String::from_utf8(bytes).map_err(|source| Error::InvalidInput {
                    message: format!("MoeResearch research skill is not valid UTF-8: {source}"),
                })?;
                let content = content.replace("../prompts/", "./prompts/");
                return Ok(Some((target.root.join("SKILL.md"), content.into_bytes())));
            }

            if source_path.starts_with("skills") {
                return Ok(None);
            }

            let prompt_path =
                source_path
                    .strip_prefix("prompts")
                    .map_err(|_| Error::InvalidInput {
                        message: format!("unexpected Claude Code asset {}", source_path.display()),
                    })?;
            Ok(Some((target.root.join("prompts").join(prompt_path), bytes)))
        }
    }
}

fn find_conflicts(planned_files: &[PlannedFile]) -> Result<Vec<Conflict>> {
    let mut conflicts = Vec::new();
    for planned in planned_files {
        if !planned.destination_path.exists() {
            continue;
        }
        if !planned.destination_path.is_file() {
            conflicts.push(Conflict {
                path: planned.destination_path.clone(),
                replaceable: false,
            });
            continue;
        }
        let current = fs::read(&planned.destination_path)
            .map_err(io_error("read existing destination file"))?;
        if current != planned.bytes {
            conflicts.push(Conflict {
                path: planned.destination_path.clone(),
                replaceable: true,
            });
        }
    }
    Ok(conflicts)
}

fn reject_blocking_conflicts(conflicts: &[Conflict], force: bool) -> Result<()> {
    let blocking = conflicts
        .iter()
        .filter(|conflict| !force || !conflict.replaceable)
        .collect::<Vec<_>>();
    if blocking.is_empty() {
        return Ok(());
    }

    Err(Error::InvalidInput {
        message: format!(
            "asset install would overwrite or replace conflicting paths: {}",
            blocking
                .iter()
                .map(|conflict| conflict.path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    })
}

fn write_planned_files(planned_files: &[PlannedFile]) -> Result<()> {
    for planned in planned_files {
        if let Some(parent) = planned.destination_path.parent() {
            fs::create_dir_all(parent).map_err(io_error("create asset destination directory"))?;
        }
        if planned.destination_path.exists() && planned.destination_path.is_file() {
            let current = fs::read(&planned.destination_path)
                .map_err(io_error("read existing destination file"))?;
            if current == planned.bytes {
                continue;
            }
        }
        write_file_via_temp(
            &planned.destination_path,
            &planned.source_path,
            &planned.bytes,
        )?;
    }
    Ok(())
}

fn write_file_via_temp(destination: &Path, source_path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = destination.parent().ok_or_else(|| Error::InvalidInput {
        message: format!("asset destination has no parent: {}", destination.display()),
    })?;
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("asset");
    let temp_path = parent.join(format!(
        ".{source_name}.moeresearch-{}.tmp",
        std::process::id()
    ));
    fs::write(&temp_path, bytes).map_err(io_error("write temporary asset file"))?;
    replace_file(&temp_path, destination).inspect_err(|_| {
        let _ = fs::remove_file(&temp_path);
    })
}

fn replace_file(temp_path: &Path, destination: &Path) -> Result<()> {
    if cfg!(windows) && destination.exists() {
        fs::remove_file(destination).map_err(io_error("remove existing asset destination file"))?;
    }
    fs::rename(temp_path, destination).map_err(io_error("replace asset destination file"))
}

fn verify_bytes_sha256(bytes: &[u8], expected: &str, label: &str) -> Result<()> {
    let actual = bytes_sha256(bytes);
    if actual == expected {
        return Ok(());
    }
    Err(Error::InvalidInput {
        message: format!("{label} checksum mismatch"),
    })
}

fn file_sha256(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(File::open(path).map_err(io_error("open file"))?);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(io_error("read file"))?;
    Ok(bytes_sha256(&bytes))
}

fn bytes_sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let hash = Sha256::digest(bytes);
    hash.iter().fold(String::new(), |mut output, byte| {
        write!(output, "{byte:02x}").expect("write to string");
        output
    })
}

fn io_error(action: &'static str) -> impl FnOnce(std::io::Error) -> Error {
    move |source| Error::Internal {
        message: format!("{action}: {source}"),
    }
}

fn network_error(source: &reqwest::Error) -> Error {
    Error::NetworkFailed {
        message: source.to_string(),
    }
}
