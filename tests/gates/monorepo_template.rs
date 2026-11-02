//! `templates/monorepo`, made the way a user makes it: `rok init shop
//! --template monorepo --org com.acme` into a scratch directory (every App
//! renamed after the project), then `rok gen` from
//! the root, which regenerates the api App, re-exports its contract
//! (`apps/api/openapi.json`, through the App's `openapi` example) and
//! emits the Clients from it (workspace ADR 0027). What the template checks
//! in must be exactly what that writes — the api App's generated files,
//! its `openapi.json`, `clients/ts`, `clients/dart` and `clients/rust`'s
//! sources (set `ROK_DATA_SQLC_UPDATE_EXPECTED=1` to rewrite them) — and the
//! project must pass its own checks: rustfmt, the lint gate, `check-arch`,
//! and `cargo test` (its contract test, its artifact test, and its API
//! tests — those driven through the generated Rust client among them —
//! against `ROK_DATA_SQLC_TEST_DATABASE_URL` when set). The Rust client
//! passes its own: rustfmt, clippy and its tests, under the template's
//! `Cargo.lock`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::common::{self, cargo};
use crate::rok_data_sqlc;

/// The monorepo template's directory.
fn template_dir() -> PathBuf {
    common::template_dir("monorepo")
}

/// `rok gen` in `dir`, which must succeed. The contract's export builds
/// the api App: it builds in the gates' shared target directory, with
/// [`cargo`]'s environment.
pub(crate) fn rok_gen_shared(dir: &Path, args: &[&str]) -> Result<String> {
    let mut command = Command::new(crate::rok_data_bin()?);
    command.arg("gen").args(args).current_dir(dir);
    for (key, value) in cargo(dir, &[]).get_envs() {
        if let Some(value) = value {
            command.env(key, value);
        }
    }
    let output = command.output().context("failed to run rok gen")?;
    ensure!(
        output.status.success(),
        "rok gen {args:?} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The project name the gates scaffold the monorepo as: init names every
/// App after it (workspace ADR 0027), so every gate checks the renamed
/// Apps.
pub(crate) const PROJECT: &str = "shop";

/// The org the gates scaffold the monorepo under.
pub(crate) const ORG: &str = "com.acme";

/// The api App's package in a scaffold named [`PROJECT`].
pub(crate) const API_PACKAGE: &str = "shop-api";

/// A scaffolded monorepo: the project directory [`PROJECT`] in a scratch
/// directory removed on drop.
pub(crate) struct Scaffold {
    _scratch: tempfile::TempDir,
    root: PathBuf,
}

impl Scaffold {
    /// The project's root.
    pub(crate) fn path(&self) -> &Path {
        &self.root
    }
}

/// `init <PROJECT> --template monorepo --org <ORG>` in a scratch
/// directory, the way a user makes the project (it copies the template's
/// `Cargo.lock`, so builds resolve as the template's do).
pub(crate) fn scaffold() -> Result<Scaffold> {
    let scratch = tempfile::TempDir::new()?;
    rok_data_sqlc(
        scratch.path(),
        &["init", PROJECT, "--template", "monorepo", "--org", ORG],
    )?;
    let root = scratch.path().join(PROJECT);
    Ok(Scaffold {
        _scratch: scratch,
        root,
    })
}

/// A project made from the monorepo template: [`scaffold`], then `rok
/// gen` from the root.
pub(crate) fn materialize() -> Result<Scaffold> {
    let project = scaffold()?;
    let dir = project.path();
    // With ROK_DATA_PATH set, the template's `rok-data = "0.5"` resolves to
    // the generator's own checkout instead of crates.io (see main.rs).
    // The patch shim goes at the workspace root.
    common::rok_data_patch_shim(dir)?;
    rok_gen_shared(dir, &[])?;
    Ok(project)
}

/// rok-data-sqlc's generated files under `root` (a build's `target/`
/// aside), by path relative to `root`, with `\n` line endings.
fn generated_files(root: &Path) -> Result<BTreeMap<PathBuf, String>> {
    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, String>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let relative = path.strip_prefix(root)?.to_path_buf();
            if path.is_dir() {
                if relative != Path::new("target") {
                    walk(root, &path, files)?;
                }
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if common::is_generated(&text) {
                files.insert(relative, text.replace("\r\n", "\n"));
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files)?;
    Ok(files)
}

/// The drift between the generated files under `sub` of the project and
/// of the template, each a line; with `ROK_DATA_SQLC_UPDATE_EXPECTED` set,
/// the template's are rewritten instead, and there is none.
fn drift_under(project: &Path, sub: &str) -> Result<Vec<String>> {
    let generated = generated_files(&project.join(sub))?;
    let template = template_dir().join(sub);
    let checked_in = generated_files(&template)?;
    if std::env::var_os("ROK_DATA_SQLC_UPDATE_EXPECTED").is_some() {
        for path in checked_in
            .keys()
            .filter(|path| !generated.contains_key(*path))
        {
            std::fs::remove_file(template.join(path))?;
        }
        for (path, text) in &generated {
            let target = template.join(path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(target, text)?;
        }
        return Ok(Vec::new());
    }
    Ok(generated
        .keys()
        .chain(checked_in.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| generated.get(*path) != checked_in.get(*path))
        .map(|path| {
            let shown = format!("{sub}/{}", common::shown(path));
            match (generated.contains_key(path), checked_in.contains_key(path)) {
                (true, false) => format!("{shown}: generated, not checked in"),
                (false, true) => format!("{shown}: checked in, no longer generated"),
                _ => format!("{shown}: differs"),
            }
        })
        .collect())
}

/// The template checks in exactly the files `rok gen` generates: the api
/// App's, from its schema and `rok-data-sqlc.toml`, and the TS and Dart
/// clients', from the contract. With `ROK_DATA_SQLC_UPDATE_EXPECTED` set, it
/// rewrites them instead.
#[test]
fn monorepo_template_checks_in_what_generation_writes() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let project = materialize()?;
    let generated = generated_files(project.path())?;
    for expected in [
        "apps/api/src/features/users/dto.rs",
        "apps/api/src/db/http/axum.rs",
        "clients/ts/src/calls.ts",
        "clients/dart/lib/src/client.dart",
        "clients/rust/src/calls.rs",
    ] {
        ensure!(
            generated.contains_key(Path::new(expected)),
            "`rok gen` generated no {expected}: {:?}",
            generated.keys()
        );
    }
    let mut drift = drift_under(project.path(), "apps/api")?;
    drift.extend(drift_under(project.path(), "clients/ts")?);
    drift.extend(drift_under(project.path(), "clients/dart")?);
    drift.extend(drift_under(project.path(), "clients/rust")?);
    ensure!(
        drift.is_empty(),
        "templates/monorepo drifted from its generation (regenerate with ROK_DATA_SQLC_UPDATE_EXPECTED=1):\n  {}",
        drift.join("\n  ")
    );
    Ok(())
}

/// The api App's README lists the dependencies and lints of its
/// generated crate; the generated crate compiles and passes its checks.
/// Its tests include `tests/client/`, which drive the API through the
/// generated `clients/rust`: against PostgreSQL when
/// `ROK_DATA_SQLC_TEST_DATABASE_URL` is set (they skip otherwise).
#[test]
fn monorepo_api_app_compiles_and_passes_checks() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let project = materialize()?;
    // `cargo test -p shop-api` from the root builds and tests the api App,
    // renamed after the project. The desktop App, the other Rust member,
    // has its own gate (`desktop_app.rs`): GPUI's build is slow and needs
    // system libraries on Linux, so the Rust matrix does not pay for it
    // here.
    let mut command = cargo(project.path(), &["test", "-p", API_PACKAGE]);
    if let Ok(url) = std::env::var("ROK_DATA_SQLC_TEST_DATABASE_URL") {
        command.env("DATABASE_URL", url);
    }
    let status = command.status()?;
    ensure!(
        status.success(),
        "`cargo test -p {API_PACKAGE}` failed: {status}"
    );
    Ok(())
}

/// The Rust client `init` emits (`clients/rust`) is a member of the root
/// workspace the template's `Cargo.lock` covers (`--locked`), formatted
/// as rustfmt formats it, clean under clippy `-D warnings` with its own
/// lints, and its tests (doc tests) pass.
#[test]
fn monorepo_rust_client_passes_its_checks() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let project = scaffold()?;
    let root = project.path();
    let checks: [&[&str]; 3] = [
        &["fmt", "-p", "api-client", "--check"],
        &[
            "clippy",
            "-p",
            "api-client",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        &["test", "-p", "api-client", "--locked"],
    ];
    for args in checks {
        let status = cargo(root, args).status()?;
        ensure!(
            status.success(),
            "`cargo {}` in {} failed: {status}",
            args.join(" "),
            common::shown(root)
        );
    }
    Ok(())
}

/// The api App's `openapi.json` is the document the App serves, as `rok
/// gen` re-exported it: a contract change re-exports it, nobody edits it.
/// The document is titled after the api App's package (`CARGO_PKG_NAME`):
/// the template's `axum-api`, the scaffold's renamed one. With
/// `ROK_DATA_SQLC_UPDATE_EXPECTED` set, it rewrites the template's copy
/// instead.
#[test]
fn monorepo_template_checks_in_the_exported_contract() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    const SHIPPED_TITLE: &str = "\"title\": \"axum-api\",";
    let renamed_title = format!("\"title\": \"{API_PACKAGE}\",");
    let project = materialize()?;
    let exported = std::fs::read_to_string(project.path().join("apps/api/openapi.json"))?
        .replace("\r\n", "\n");
    ensure!(
        exported.contains(&renamed_title),
        "the scaffold's contract is not titled `{API_PACKAGE}`"
    );
    let exported = exported.replacen(&renamed_title, SHIPPED_TITLE, 1);
    let artifact = template_dir().join("apps/api/openapi.json");
    if std::env::var_os("ROK_DATA_SQLC_UPDATE_EXPECTED").is_some() {
        std::fs::write(&artifact, &exported)?;
        return Ok(());
    }
    let checked_in = std::fs::read_to_string(&artifact)?.replace("\r\n", "\n");
    ensure!(
        checked_in == exported,
        "templates/monorepo/apps/api/openapi.json is not what `rok gen` exports \
         (regenerate with ROK_DATA_SQLC_UPDATE_EXPECTED=1; never edit it by hand)"
    );
    Ok(())
}
