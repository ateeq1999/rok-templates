//! The `apps/web` App of the monorepo template, made the way a user makes
//! it: `rok init shop --template monorepo` into a scratch directory, then, in
//! the scaffolded `apps/web`, the web App's own checks with its own
//! tooling — `bun install --frozen-lockfile` (the template's `bun.lock`
//! must match its `package.json`), `bun run typecheck` and `bun run test`
//! (the App's vitest suite; `bun test`, bun's own runner, is not it).
//! The App calls the API through the generated `clients/ts`, which init
//! emits beside it; a contract change is followed through `rok gen` and
//! the same checks. Skips with a printed note when bun is absent.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::common;

/// The `bun` to run: the one on `PATH`, else the one bun's installer puts
/// under `~/.bun/bin`; `None` when neither runs.
fn bun() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| {
            PathBuf::from(home).join(format!(".bun/bin/bun{}", std::env::consts::EXE_SUFFIX))
        });
    std::iter::once(PathBuf::from("bun"))
        .chain(home)
        .find(|bun| {
            Command::new(bun)
                .arg("--version")
                .output()
                .is_ok_and(|output| output.status.success())
        })
}

/// The major version of the `node` on `PATH`, when there is one.
fn node_major() -> Option<u32> {
    let output = Command::new("node").arg("--version").output().ok()?;
    let version = String::from_utf8(output.stdout).ok()?;
    version
        .trim()
        .strip_prefix('v')?
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// `bun <args>` in `dir`, which must succeed.
fn run(bun: &Path, dir: &Path, args: &[&str]) -> Result<()> {
    let mut command = Command::new(bun);
    command.args(args).current_dir(dir);
    // Node 25 turned Web Storage on by default: its global `localStorage`
    // (unusable without `--localstorage-file`) shadows jsdom's in vitest,
    // and the App's session tests read `window.localStorage`. Turned off,
    // jsdom's storage is the one the tests see, as on the Node the App was
    // written for.
    if node_major().is_some_and(|major| major >= 25) {
        command.env("NODE_OPTIONS", "--no-experimental-webstorage");
    }
    let output = command.output().context("failed to run bun")?;
    ensure!(
        output.status.success(),
        "bun {args:?} in {} failed:\n{}{}",
        common::shown(dir),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// The scaffold writes the web App's binary files byte for byte, and its
/// `.env.example`.
#[test]
fn monorepo_scaffold_writes_the_web_app_whole() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let project = crate::monorepo_template::scaffold()?;
    let template = common::template_dir("monorepo").join("apps/web");
    let web = project.path().join("apps/web");
    for file in ["public/favicon.ico", "public/logo.png"] {
        ensure!(
            std::fs::read(web.join(file))? == std::fs::read(template.join(file))?,
            "the scaffolded apps/web/{file} differs from the template's"
        );
    }
    for file in [
        ".env.example",
        "package.json",
        "bun.lock",
        "vitest.config.ts",
    ] {
        ensure!(
            web.join(file).is_file(),
            "the scaffold has no apps/web/{file}"
        );
    }
    Ok(())
}

/// The gate: scaffold the monorepo, then install the web App's locked
/// dependencies, typecheck it and run its tests.
#[test]
fn monorepo_web_app_installs_typechecks_and_tests() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(bun) = bun() else {
        eprintln!(
            "note: skipping the web App gate: `bun` is not installed (https://bun.sh installs it)"
        );
        return Ok(());
    };
    let project = crate::monorepo_template::scaffold()?;
    let web = project.path().join("apps/web");
    run(&bun, &web, &["install", "--frozen-lockfile"])?;
    run(&bun, &web, &["run", "typecheck"])?;
    run(&bun, &web, &["run", "test"])?;
    Ok(())
}

/// A contract change round-trips through regeneration and the gate: a
/// change to the api App's source is re-exported by `rok gen` into
/// `openapi.json`, regenerates `clients/ts`, leaves `rok gen check` clean,
/// and the web App installs, typechecks and passes its tests with the new
/// client (workspace ADR 0027).
#[test]
fn monorepo_contract_change_round_trips_through_the_web_gate() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(bun) = bun() else {
        eprintln!(
            "note: skipping the contract round trip: `bun` is not installed (https://bun.sh installs it)"
        );
        return Ok(());
    };
    let project = crate::monorepo_template::materialize()?;
    let dir = project.path();
    let before = std::fs::read_to_string(dir.join("clients/ts/src/types.ts"))?;

    // The change: a field of the API's `TotpSetupResponse` documented anew.
    // The secret's description is the only one of its words in the api
    // App's sources and in its document.
    const OLD: &str = "Base32-encoded; lets the user type the secret in by hand if they\n    /// cannot scan the QR code.";
    const NEW: &str = "The secret, base32-encoded, for typing in by hand.";
    let dto = dir.join("apps/api/src/features/two_factor/dto.rs");
    let source = std::fs::read_to_string(&dto)?.replace("\r\n", "\n");
    ensure!(
        source.contains(OLD),
        "the api App's two-factor DTO no longer says `{OLD}`: pick another change"
    );
    std::fs::write(&dto, source.replace(OLD, NEW))?;

    crate::monorepo_template::rok_gen_shared(dir, &[])?;
    let contract = std::fs::read_to_string(dir.join("apps/api/openapi.json"))?;
    ensure!(
        contract.contains(NEW),
        "`rok gen` did not re-export the changed contract"
    );
    let after = std::fs::read_to_string(dir.join("clients/ts/src/types.ts"))?;
    ensure!(
        after != before && after.contains(NEW),
        "`rok gen` did not regenerate clients/ts from the changed contract"
    );
    crate::monorepo_template::rok_gen_shared(dir, &["check"])?;

    let web = dir.join("apps/web");
    run(&bun, &web, &["install", "--frozen-lockfile"])?;
    run(&bun, &web, &["run", "typecheck"])?;
    run(&bun, &web, &["run", "test"])?;
    Ok(())
}
