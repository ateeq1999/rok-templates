//! The templates' gates: each template made into a project by a `rok-data`
//! generator (`sqlc init --template`, then generation), then checked the way
//! the user's project would be — drift against what is checked in, `cargo`
//! checks and tests, and the monorepo's web, mobile, desktop and Dart gates.
//!
//! The generator is the `rok-data` binary named by `ROK_DATA_BIN`. It must
//! be built against this checkout, because `init` copies the templates
//! embedded in the binary, not these files:
//!
//! ```console
//! $ cd ../rok-data   # the rok-data checkout at the pin in rok-data.pin
//! $ cargo build -p rok-data-cli --config 'patch.crates-io.rok-templates.path="../rok-templates"'
//! ```
//!
//! `ROK_DATA_PATH`, when set, points the generated projects' `rok-data`
//! dependency at that checkout's `crates/rok-data` instead of crates.io, so
//! the runtime matches the generator. Without `ROK_DATA_BIN` every gate that
//! needs it prints why and passes; a binary embedding other templates fails.

mod common;

mod actix_template;
mod axum_template;
mod dart_client;
mod desktop_app;
mod mobile_app;
mod monorepo_template;
mod web_app;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, ensure};

/// The generator `ROK_DATA_BIN` names, once checked to embed this
/// checkout's templates.
fn rok_data_bin() -> Result<PathBuf> {
    static CHECKED: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CHECKED
        .get_or_init(|| check_generator().map_err(|e| format!("{e:#}")))
        .clone()
        .map_err(|e| anyhow!(e))
}

fn check_generator() -> Result<PathBuf> {
    let bin = PathBuf::from(std::env::var_os("ROK_DATA_BIN").context("ROK_DATA_BIN is not set")?);
    // `init` copies the embedded templates; one file it copies verbatim
    // tells whether they are this checkout's.
    let project = tempfile::TempDir::new()?;
    let output = Command::new(&bin)
        .args(["sqlc", "init", "--template", "axum"])
        .current_dir(project.path())
        .output()
        .with_context(|| format!("failed to run {}", bin.display()))?;
    ensure!(
        output.status.success(),
        "{} sqlc init failed:\n{}",
        bin.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let copied = std::fs::read(project.path().join("AGENTS.md"))?;
    let here = std::fs::read(common::template_dir("axum-api").join("AGENTS.md"))?;
    ensure!(
        copied == here,
        "{} embeds templates other than this checkout's; build it with \
         --config 'patch.crates-io.rok-templates.path=\"…/rok-templates\"' (see tests/gates/main.rs)",
        bin.display()
    );
    Ok(bin)
}

/// Whether the gate calling it must skip: no `ROK_DATA_BIN`, said on
/// stderr so the skip is never silent.
pub(crate) fn generator_missing() -> bool {
    if std::env::var_os("ROK_DATA_BIN").is_some() {
        return false;
    }
    eprintln!(
        "note: skipping: set ROK_DATA_BIN to a rok-data binary built against this checkout \
         (see tests/gates/main.rs)"
    );
    true
}

/// Runs `rok-data sqlc <args>` in `dir`, which must succeed, and returns its
/// stderr.
fn rok_data_sqlc(dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(rok_data_bin()?)
        .arg("sqlc")
        .args(args)
        .current_dir(dir)
        .output()
        .context("failed to run rok-data sqlc")?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    ensure!(
        output.status.success(),
        "rok-data sqlc {args:?} failed:\n{}{stderr}",
        String::from_utf8_lossy(&output.stdout),
    );
    Ok(stderr)
}
