#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use tempfile::TempDir;

const BUILD_SCRIPT: &str = include_str!("../scripts/build-release.sh");
const FAKE_CARGO: &str = r#"#!/usr/bin/env bash
set -eu
printf 'application=%s\nkey=%s\nindex=%s\n' "$ALGOLIA_APPLICATION_ID" "$ALGOLIA_SEARCH_ONLY_API_KEY" "$ALGOLIA_SEARCH_INDEX" > "$CAPTURE"
if printenv UNRELATED >/dev/null 2>&1; then
  exit 9
fi
mkdir -p target/release
: > target/release/alfred_flutter_docs
chmod +x target/release/alfred_flutter_docs
"#;

fn fixture(dotenv: Option<&str>) -> Result<(TempDir, PathBuf, PathBuf)> {
    let directory = tempfile::tempdir()?;
    let bin = directory.path().join("bin");
    fs::create_dir(&bin)?;
    let script = directory.path().join("build-release.sh");
    fs::write(&script, BUILD_SCRIPT)?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    let cargo = bin.join("cargo");
    fs::write(&cargo, FAKE_CARGO)?;
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755))?;
    if let Some(contents) = dotenv {
        fs::write(directory.path().join(".env"), contents)?;
    }
    Ok((directory, bin, script))
}

fn run(
    directory: &Path,
    bin: &Path,
    script: &Path,
    runtime: &[(&str, &str)],
) -> Result<std::process::Output> {
    let capture = directory.join("capture.txt");
    let path = format!("{}:/usr/bin:/bin", bin.display());
    let mut command = Command::new("/bin/bash");
    command
        .arg(script)
        .current_dir(directory)
        .env_clear()
        .env("PATH", path)
        .env("CAPTURE", &capture);
    for (name, value) in runtime {
        command.env(name, value);
    }
    command.output().context("run fake release build")
}

fn capture(directory: &Path) -> Result<String> {
    fs::read_to_string(directory.join("capture.txt")).map_err(Into::into)
}

#[test]
fn complete_runtime_values_skip_malformed_dotenv_and_unrelated_exports() -> Result<()> {
    let (directory, bin, script) =
        fixture(Some("BROKEN=\"unterminated\nUNRELATED=dotenv-secret\n"))?;
    let output = run(
        directory.path(),
        &bin,
        &script,
        &[
            ("ALGOLIA_APPLICATION_ID", "runtime-app"),
            ("ALGOLIA_SEARCH_ONLY_API_KEY", "runtime-key"),
            ("ALGOLIA_SEARCH_INDEX", "runtime-index"),
        ],
    )?;
    assert!(
        output.status.success(),
        "stderr: {} stdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let output = capture(directory.path())?;
    assert!(output.contains("application=runtime-app"));
    assert!(output.contains("key=runtime-key"));
    assert!(output.contains("index=runtime-index"));
    Ok(())
}

#[test]
fn partial_runtime_values_use_only_missing_dotenv_values() -> Result<()> {
    let (directory, bin, script) = fixture(Some(
        "ALGOLIA_SEARCH_ONLY_API_KEY=dotenv-key\nALGOLIA_SEARCH_INDEX=dotenv-index\nUNRELATED=secret\n",
    ))?;
    let output = run(
        directory.path(),
        &bin,
        &script,
        &[("ALGOLIA_APPLICATION_ID", "runtime-app")],
    )?;
    assert!(
        output.status.success(),
        "stderr: {} stdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let output = capture(directory.path())?;
    assert!(output.contains("application=runtime-app"));
    assert!(output.contains("key=dotenv-key"));
    assert!(output.contains("index=dotenv-index"));
    Ok(())
}

#[test]
fn empty_runtime_and_dotenv_values_are_not_replaced() -> Result<()> {
    let (directory, bin, script) = fixture(Some(
        "ALGOLIA_APPLICATION_ID=dotenv-app\nALGOLIA_SEARCH_ONLY_API_KEY=dotenv-key\nALGOLIA_SEARCH_INDEX=dotenv-index\n",
    ))?;
    let output = run(
        directory.path(),
        &bin,
        &script,
        &[
            ("ALGOLIA_APPLICATION_ID", ""),
            ("ALGOLIA_SEARCH_ONLY_API_KEY", "runtime-key"),
            ("ALGOLIA_SEARCH_INDEX", "runtime-index"),
        ],
    )?;
    assert!(!output.status.success());
    assert!(!directory.path().join("capture.txt").exists());

    let (directory, bin, script) = fixture(Some(
        "ALGOLIA_APPLICATION_ID=dotenv-app\nALGOLIA_SEARCH_ONLY_API_KEY=\nALGOLIA_SEARCH_INDEX=dotenv-index\n",
    ))?;
    let output = run(
        directory.path(),
        &bin,
        &script,
        &[("ALGOLIA_APPLICATION_ID", "runtime-app")],
    )?;
    assert!(!output.status.success());
    assert!(!directory.path().join("capture.txt").exists());
    Ok(())
}

#[test]
fn dotenv_failures_after_assignments_and_early_exit_abort_build() -> Result<()> {
    for dotenv in [
        "ALGOLIA_APPLICATION_ID=app\nALGOLIA_SEARCH_ONLY_API_KEY=key\nALGOLIA_SEARCH_INDEX=index\nfalse\n",
        "ALGOLIA_APPLICATION_ID=app\nALGOLIA_SEARCH_ONLY_API_KEY=key\nALGOLIA_SEARCH_INDEX=index\nexit 7\n",
    ] {
        let (directory, bin, script) = fixture(Some(dotenv))?;
        let output = run(directory.path(), &bin, &script, &[])?;
        assert!(!output.status.success());
        assert!(!directory.path().join("capture.txt").exists());
    }
    Ok(())
}

#[test]
fn dotenv_is_not_discovered_through_path() -> Result<()> {
    let (directory, bin, script) = fixture(None)?;
    let path_dotenv = directory.path().join("path-dotenv");
    fs::create_dir(&path_dotenv)?;
    fs::write(
        path_dotenv.join(".env"),
        "ALGOLIA_APPLICATION_ID=path-app\nALGOLIA_SEARCH_ONLY_API_KEY=path-key\nALGOLIA_SEARCH_INDEX=path-index\n",
    )?;
    let path = format!("{}:{}:/usr/bin:/bin", path_dotenv.display(), bin.display());
    let capture = directory.path().join("capture.txt");
    let output = Command::new("/bin/bash")
        .arg(&script)
        .current_dir(directory.path())
        .env_clear()
        .env("PATH", path)
        .env("CAPTURE", capture)
        .output()?;
    assert!(!output.status.success());
    assert!(!directory.path().join("capture.txt").exists());
    Ok(())
}

#[test]
fn fixture_has_no_unexpected_error() -> Result<()> {
    let (directory, bin, script) = fixture(None)?;
    let output = run(
        directory.path(),
        &bin,
        &script,
        &[
            ("ALGOLIA_APPLICATION_ID", "app"),
            ("ALGOLIA_SEARCH_ONLY_API_KEY", "key"),
            ("ALGOLIA_SEARCH_INDEX", "index"),
        ],
    )?;
    if !output.status.success() {
        return Err(anyhow!(
            "fake cargo failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
