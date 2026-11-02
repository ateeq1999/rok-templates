//! The monorepo's generated Dart client (`clients/dart`, workspace ADR
//! 0027), made the way a user makes it: `rok init shop --template monorepo`
//! into a scratch directory emits it from the scaffolded contract; then,
//! in it, the package's own checks with Dart's own tooling — `dart pub
//! get`, `dart analyze --fatal-infos` (its `analysis_options.yaml`'s
//! strict modes and lints) and `dart test`, running `dart_client_test.dart`
//! (beside this file) against the contract's operations over a mock HTTP
//! client. Skips with a printed note when `dart` is absent.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::common;

/// The `dart` to run: the one on `PATH` (on Windows, Flutter's
/// `dart.bat` too); `None` when none runs.
fn dart() -> Option<PathBuf> {
    let candidates: &[&str] = if cfg!(windows) {
        &["dart", "dart.bat"]
    } else {
        &["dart"]
    };
    candidates.iter().map(PathBuf::from).find(|dart| {
        Command::new(dart)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    })
}

/// `dart <args>` in `dir`, and whether it succeeded, with its output.
fn run(dart: &Path, dir: &Path, args: &[&str]) -> Result<(bool, String)> {
    let output = Command::new(dart)
        .args(args)
        .current_dir(dir)
        .output()
        .context("failed to run dart")?;
    Ok((
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    ))
}

/// `dart <args>` in `dir`, which must succeed.
fn check(dart: &Path, dir: &Path, args: &[&str]) -> Result<()> {
    let (ok, output) = run(dart, dir, args)?;
    ensure!(
        ok,
        "dart {args:?} in {} failed:\n{output}",
        common::shown(dir)
    );
    Ok(())
}

/// The gate: scaffold the monorepo, then resolve, analyze and test the
/// Dart client it emitted.
#[test]
fn monorepo_dart_client_analyzes_and_tests() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(dart) = dart() else {
        eprintln!(
            "note: skipping the Dart client gate: `dart` is not installed \
             (https://dart.dev/get-dart, or Flutter, installs it)"
        );
        return Ok(());
    };
    let project = crate::monorepo_template::scaffold()?;
    let client = project.path().join("clients/dart");
    ensure!(
        client.join("pubspec.yaml").is_file(),
        "init emitted no clients/dart/pubspec.yaml"
    );
    std::fs::create_dir_all(client.join("test"))?;
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/gates/dart_client_test.dart"),
        client.join("test/client_test.dart"),
    )?;
    // Resolve from pub.dev, or from the local cache when offline.
    let (online, output) = run(&dart, &client, &["pub", "get"])?;
    if !online {
        let (offline, offline_output) = run(&dart, &client, &["pub", "get", "--offline"])?;
        ensure!(
            offline,
            "dart pub get failed online and offline:\n{output}\n{offline_output}"
        );
    }
    check(&dart, &client, &["analyze", "--fatal-infos"])?;
    check(&dart, &client, &["test"])?;
    Ok(())
}
