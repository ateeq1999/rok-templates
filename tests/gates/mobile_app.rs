//! The `apps/mobile` App of the monorepo template (workspace ADR 0027),
//! made the way a user makes it: `rok init shop --template monorepo` into a
//! scratch directory writes it whole — its platform folders' binary files
//! byte for byte, its `.env.example`, and Firebase's placeholders, never a
//! real project's configuration. Then, in the scaffolded `apps/mobile`,
//! the App's own checks with Flutter's tooling: `flutter pub get
//! --enforce-lockfile` (its pinned `pubspec.lock`, never a new resolution; it
//! resolves the generated `clients/dart` by path), `flutter analyze` and
//! `flutter test`. Skips with a printed note when `flutter` is absent.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::common;

/// The `flutter` to run: the one on `PATH` (on Windows, its
/// `flutter.bat`); `None` when none runs.
fn flutter() -> Option<PathBuf> {
    let candidates: &[&str] = if cfg!(windows) {
        &["flutter.bat", "flutter"]
    } else {
        &["flutter"]
    };
    candidates.iter().map(PathBuf::from).find(|flutter| {
        Command::new(flutter)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    })
}

/// `flutter <args>` in `dir`, and whether it succeeded, with its output.
fn run(flutter: &Path, dir: &Path, args: &[&str]) -> Result<(bool, String)> {
    let output = Command::new(flutter)
        .args(args)
        .current_dir(dir)
        // No analytics prompt or banner in a test's output.
        .env("FLUTTER_SUPPRESS_ANALYTICS", "true")
        .env("CI", "true")
        .output()
        .context("failed to run flutter")?;
    Ok((
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    ))
}

/// `flutter <args>` in `dir`, which must succeed.
fn check(flutter: &Path, dir: &Path, args: &[&str]) -> Result<()> {
    let (ok, output) = run(flutter, dir, args)?;
    ensure!(
        ok,
        "flutter {args:?} in {} failed:\n{output}",
        common::shown(dir)
    );
    Ok(())
}

/// The gate: scaffold the monorepo, then resolve, analyze and test the
/// mobile App, which calls the API through the `clients/dart` init
/// emitted beside it.
#[test]
fn monorepo_mobile_app_analyzes_and_tests() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(flutter) = flutter() else {
        eprintln!(
            "note: skipping the mobile App gate: `flutter` is not installed \
             (https://docs.flutter.dev/get-started/install installs it)"
        );
        return Ok(());
    };
    let project = crate::monorepo_template::scaffold()?;
    let mobile = project.path().join("apps/mobile");
    // Install what `pubspec.lock` pins (it fails rather than resolve anew),
    // from pub.dev, or from the local cache when offline.
    let (online, output) = run(&flutter, &mobile, &["pub", "get", "--enforce-lockfile"])?;
    if !online {
        let (offline, offline_output) = run(
            &flutter,
            &mobile,
            &["pub", "get", "--offline", "--enforce-lockfile"],
        )?;
        ensure!(
            offline,
            "flutter pub get --enforce-lockfile failed online and offline:\n{output}\n{offline_output}"
        );
    }
    check(&flutter, &mobile, &["analyze", "--fatal-infos"])?;
    check(&flutter, &mobile, &["test"])?;
    Ok(())
}

/// The scaffold writes the mobile App's binary files byte for byte, its
/// `.env.example`, and the Firebase placeholders.
#[test]
fn monorepo_scaffold_writes_the_mobile_app_whole() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let project = crate::monorepo_template::scaffold()?;
    let template = common::template_dir("monorepo").join("apps/mobile");
    let mobile = project.path().join("apps/mobile");
    for file in [
        "android/app/src/main/res/mipmap-hdpi/ic_launcher.png",
        "ios/Runner/Assets.xcassets/AppIcon.appiconset/Icon-App-1024x1024@1x.png",
        "web/favicon.png",
    ] {
        ensure!(
            std::fs::read(mobile.join(file))? == std::fs::read(template.join(file))?,
            "the scaffolded apps/mobile/{file} differs from the template's"
        );
    }
    for file in [
        ".env.example",
        "pubspec.yaml",
        "pubspec.lock",
        "analysis_options.yaml",
    ] {
        ensure!(
            mobile.join(file).is_file(),
            "the scaffold has no apps/mobile/{file}"
        );
    }
    for file in [
        "android/app/google-services.json",
        "ios/Runner/GoogleService-Info.plist",
    ] {
        let text = std::fs::read_to_string(mobile.join(file))?;
        ensure!(
            text.contains("your-firebase-project"),
            "apps/mobile/{file} is not the Firebase placeholder"
        );
    }
    Ok(())
}
