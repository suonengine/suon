use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use serde_json::Value;

const PRODUCT: &str = "suon";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err("usage: cargo build-artifacts [cargo build args]".to_string());
    };

    match command.as_str() {
        "build-artifacts" => build_artifacts(args.collect()),
        _ => Err(format!("unknown build command: {command}")),
    }
}

fn build_artifacts(args: Vec<String>) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let profile = build_profile(&args);
    let explicit_target = explicit_target(&args);
    let channel = channel(&args, profile);
    let raw_cargo_args = filtered_cargo_args(&args);
    let metadata = workspace_metadata(&workspace_root)?;
    let package = root_package(&metadata, &workspace_root)?;
    let selected_targets = selected_targets(package, &raw_cargo_args);
    let cargo_args = ensure_target_selection(raw_cargo_args, &selected_targets);

    let mut cargo = Command::new("cargo");
    cargo.current_dir(&workspace_root);
    cargo.arg("build");
    cargo.args(&cargo_args);

    let status = cargo
        .status()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;

    if !status.success() {
        return Err(format!("cargo build failed with status {status}"));
    }

    let target_triple = explicit_target
        .clone()
        .or_else(|| host_target_triple(&workspace_root));

    let platform = target_triple
        .as_deref()
        .map(normalize_platform)
        .unwrap_or_else(host_platform);

    let arch = target_triple
        .as_deref()
        .map(normalize_arch)
        .unwrap_or_else(host_arch);

    let runtime = target_triple.as_deref().and_then(runtime_variant);
    let version = format!(
        "v{}",
        package
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| "failed to read package version from cargo metadata".to_string())?
    );

    let commit = git_short_commit(&workspace_root);

    if selected_targets.is_empty() {
        return Err("no binary or example targets were selected for renaming".to_string());
    }

    for target in selected_targets {
        let source = compiled_artifact_path(
            &workspace_root,
            explicit_target.as_deref(),
            profile,
            &platform,
            &target,
        );

        if !source.exists() {
            return Err(format!(
                "built executable not found at {}",
                source.display()
            ));
        }

        let artifact_name = artifact_name(
            &target.name,
            &version,
            &platform,
            &arch,
            runtime.as_deref(),
            commit.as_deref(),
            &channel,
        );

        let destination = source.with_file_name(executable_name(&artifact_name, &platform));

        if source.exists() {
            rename_artifact(&source, &destination)?;
            println!("Renamed to {}", destination.display());
            continue;
        }

        if destination.exists() {
            println!("Already named {}", destination.display());
            continue;
        }

        return Err(format!(
            "built executable not found at {}",
            source.display()
        ));
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .ancestors()
        .skip(1)
        .find(|path| path.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| "failed to resolve workspace root".to_string())
}

fn build_profile(args: &[String]) -> &'static str {
    if args.iter().any(|arg| arg == "--release") {
        "release"
    } else {
        "debug"
    }
}

fn explicit_target(args: &[String]) -> Option<String> {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(target) = arg.strip_prefix("--target=") {
            return Some(target.to_string());
        }

        if arg == "--target" {
            return args.get(index + 1).cloned();
        }
        index += 1;
    }
    None
}

fn channel(args: &[String], profile: &str) -> String {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(value) = arg.strip_prefix("--channel=") {
            return normalize_channel(value);
        }

        if arg == "--channel"
            && let Some(value) = args.get(index + 1)
        {
            return normalize_channel(value);
        }
        index += 1;
    }

    if let Ok(value) = env::var("SUON_RELEASE_CHANNEL")
        && !value.is_empty()
    {
        return normalize_channel(&value);
    }

    profile.to_string()
}

fn filtered_cargo_args(args: &[String]) -> Vec<String> {
    let mut filtered = Vec::new();
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];

        if arg == "--channel" {
            index += 2;
            continue;
        }

        if arg.starts_with("--channel=") {
            index += 1;
            continue;
        }

        filtered.push(arg.clone());
        index += 1;
    }

    filtered
}

fn compiled_artifact_path(
    workspace_root: &Path,
    target_triple: Option<&str>,
    profile: &str,
    platform: &str,
    target: &Target,
) -> PathBuf {
    let mut path = workspace_root.join("target");
    if let Some(target_triple) = target_triple {
        path.push(target_triple);
    }
    path.push(profile);
    if target.kind == "example" {
        path.push("examples");
    }
    path.push(executable_name(&target.name, platform));
    path
}

fn executable_name(base_name: &str, platform: &str) -> OsString {
    if platform == "win" {
        OsString::from(format!("{base_name}.exe"))
    } else {
        OsString::from(base_name)
    }
}

fn artifact_name(
    component: &str,
    version: &str,
    platform: &str,
    arch: &str,
    runtime: Option<&str>,
    commit: Option<&str>,
    channel: &str,
) -> String {
    let mut parts = vec![
        PRODUCT.to_string(),
        component.to_string(),
        version.to_string(),
        platform.to_string(),
        arch.to_string(),
    ];

    if let Some(runtime) = runtime {
        parts.push(runtime.to_string());
    }
    if let Some(commit) = commit {
        parts.push(commit.to_string());
    }
    parts.push(channel.to_string());

    parts.join("-")
}

fn rename_artifact(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_file(destination).map_err(|error| {
            format!(
                "failed to remove previous executable at {}: {error}",
                destination.display()
            )
        })?;
    }

    fs::rename(source, destination).map_err(|error| {
        format!(
            "failed to rename executable from {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;

    rename_windows_pdb(source, destination)?;
    Ok(())
}

fn rename_windows_pdb(source: &Path, destination: &Path) -> Result<(), String> {
    if source.extension().and_then(|ext| ext.to_str()) != Some("exe") {
        return Ok(());
    }

    let source_pdb = source.with_extension("pdb");
    if !source_pdb.exists() {
        return Ok(());
    }

    let destination_pdb = destination.with_extension("pdb");
    if destination_pdb.exists() {
        fs::remove_file(&destination_pdb).map_err(|error| {
            format!(
                "failed to remove previous debug symbols at {}: {error}",
                destination_pdb.display()
            )
        })?;
    }

    fs::rename(&source_pdb, &destination_pdb).map_err(|error| {
        format!(
            "failed to rename debug symbols from {} to {}: {error}",
            source_pdb.display(),
            destination_pdb.display()
        )
    })?;

    Ok(())
}

fn workspace_metadata(workspace_root: &Path) -> Result<Value, String> {
    let output = Command::new("cargo")
        .current_dir(workspace_root)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|error| format!("failed to run cargo metadata: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed with status {}",
            output.status
        ));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse cargo metadata output: {error}"))
}

fn root_package<'a>(metadata: &'a Value, workspace_root: &Path) -> Result<&'a Value, String> {
    let root_manifest = normalize_path_for_compare(&workspace_root.join("Cargo.toml"));

    metadata
        .get("packages")
        .and_then(Value::as_array)
        .and_then(|packages| {
            packages.iter().find(|package| {
                package
                    .get("manifest_path")
                    .and_then(Value::as_str)
                    .map(normalize_str_path_for_compare)
                    .map(|path| path == root_manifest)
                    .unwrap_or(false)
            })
        })
        .ok_or_else(|| "failed to locate root package in cargo metadata".to_string())
}

fn selected_targets(package: &Value, args: &[String]) -> Vec<Target> {
    let available = package
        .get("targets")
        .and_then(Value::as_array)
        .map(|targets| {
            targets
                .iter()
                .filter_map(Target::from_metadata)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut selected_bins = Vec::new();
    let mut selected_examples = Vec::new();
    let mut include_all_bins = false;
    let mut include_all_examples = false;
    let mut include_all_targets = false;

    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];

        if let Some(name) = arg.strip_prefix("--bin=") {
            selected_bins.push(name.to_string());
        } else if arg == "--bin" {
            if let Some(name) = args.get(index + 1) {
                selected_bins.push(name.clone());
                index += 1;
            }
        } else if arg == "--bins" {
            include_all_bins = true;
        } else if let Some(name) = arg.strip_prefix("--example=") {
            selected_examples.push(name.to_string());
        } else if arg == "--example" {
            if let Some(name) = args.get(index + 1) {
                selected_examples.push(name.clone());
                index += 1;
            }
        } else if arg == "--examples" {
            include_all_examples = true;
        } else if arg == "--all-targets" {
            include_all_targets = true;
        }

        index += 1;
    }

    let has_explicit_selection = include_all_bins
        || include_all_examples
        || include_all_targets
        || !selected_bins.is_empty()
        || !selected_examples.is_empty();

    available
        .into_iter()
        .filter(|target| {
            if include_all_targets {
                return target.kind == "bin" || target.kind == "example";
            }
            if include_all_bins && target.kind == "bin" {
                return true;
            }
            if include_all_examples && target.kind == "example" {
                return true;
            }
            if target.kind == "bin" && selected_bins.iter().any(|name| name == &target.name) {
                return true;
            }
            if target.kind == "example" && selected_examples.iter().any(|name| name == &target.name)
            {
                return true;
            }

            !has_explicit_selection && (target.kind == "bin" || target.kind == "example")
        })
        .collect()
}

fn ensure_target_selection(args: Vec<String>, selected_targets: &[Target]) -> Vec<String> {
    if has_explicit_target_selection(&args) || selected_targets.is_empty() {
        return args;
    }

    let mut enriched = args;
    for target in selected_targets {
        match target.kind.as_str() {
            "bin" => {
                enriched.push("--bin".to_string());
                enriched.push(target.name.clone());
            }
            "example" => {
                enriched.push("--example".to_string());
                enriched.push(target.name.clone());
            }
            _ => {}
        }
    }

    enriched
}

fn has_explicit_target_selection(args: &[String]) -> bool {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--bin"
            || arg == "--bins"
            || arg == "--example"
            || arg == "--examples"
            || arg == "--all-targets"
            || arg.starts_with("--bin=")
            || arg.starts_with("--example=")
        {
            return true;
        }
        index += 1;
    }

    false
}

fn host_target_triple(workspace_root: &Path) -> Option<String> {
    let output = Command::new("rustc")
        .current_dir(workspace_root)
        .arg("-vV")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(ToString::to_string))
}

fn normalize_arch(target: &str) -> String {
    let arch = target.split('-').next().unwrap_or(target);
    match arch {
        "x86_64" => "x64".to_string(),
        "aarch64" => "arm64".to_string(),
        "arm" | "armv7" | "armv7l" => "arm".to_string(),
        "i586" | "i686" | "x86" => "x86".to_string(),
        other => other.to_string(),
    }
}

fn normalize_platform(target: &str) -> String {
    if target.contains("windows") {
        "win".to_string()
    } else if target.contains("linux") {
        "linux".to_string()
    } else if target.contains("darwin") || target.contains("apple") || target.contains("macos") {
        "macos".to_string()
    } else {
        target.to_string()
    }
}

fn runtime_variant(target: &str) -> Option<String> {
    if target.contains("musl") {
        Some("musl".to_string())
    } else if target.contains("msvc") {
        Some("msvc".to_string())
    } else if target.contains("gnu") {
        Some("gnu".to_string())
    } else {
        None
    }
}

fn git_short_commit(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let short = String::from_utf8(output.stdout).ok()?;
    let short = short.trim();
    if short.is_empty() {
        None
    } else {
        Some(format!("g{short}"))
    }
}

fn host_arch() -> String {
    match env::consts::ARCH {
        "x86_64" => "x64".to_string(),
        "aarch64" => "arm64".to_string(),
        "arm" => "arm".to_string(),
        "x86" => "x86".to_string(),
        other => other.to_string(),
    }
}

fn host_platform() -> String {
    match env::consts::OS {
        "windows" => "win".to_string(),
        "linux" => "linux".to_string(),
        "macos" => "macos".to_string(),
        other => other.to_string(),
    }
}

fn normalize_channel(channel: &str) -> String {
    match channel.trim().to_ascii_lowercase().as_str() {
        "nightly" => "nightly".to_string(),
        "beta" => "beta".to_string(),
        "debug" | "dev" => "debug".to_string(),
        "release" | "relreease" => "release".to_string(),
        other => other.to_string(),
    }
}

#[derive(Clone)]
struct Target {
    name: String,
    kind: String,
}

impl Target {
    fn from_metadata(target: &Value) -> Option<Self> {
        let name = target.get("name")?.as_str()?.to_string();
        let kind = target
            .get("kind")?
            .as_array()?
            .first()?
            .as_str()?
            .to_string();

        if kind == "bin" || kind == "example" {
            Some(Self { name, kind })
        } else {
            None
        }
    }
}

fn normalize_path_for_compare(path: &Path) -> String {
    normalize_str_path_for_compare(&path.to_string_lossy())
}

fn normalize_str_path_for_compare(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}
