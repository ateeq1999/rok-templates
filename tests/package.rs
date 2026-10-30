//! What `cargo package` ships: every file of every template, so
//! `init --template` from `cargo install` copies what it does from the
//! repository, and no template's build output.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, ensure};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn packaged_files() -> Result<Vec<String>> {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["package", "--list", "--allow-dirty", "--offline"])
        .current_dir(root())
        .output()
        .context("failed to run cargo package --list")?;
    ensure!(
        output.status.success(),
        "cargo package --list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?
        .lines()
        .map(|line| line.replace('\\', "/"))
        .collect())
}

#[test]
fn package_ships_the_embedding_and_licenses_but_no_build_output() -> Result<()> {
    let files = packaged_files()?;
    for required in [
        "Cargo.toml",
        "README.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "src/lib.rs",
        "build.rs",
        "templates/actix-api/rok-data-sqlc.toml",
        "templates/axum-api/Cargo.toml.template",
    ] {
        ensure!(
            files.iter().any(|file| file == required),
            "{required} missing from the package: {files:?}"
        );
    }
    ensure!(
        !files.iter().any(|file| file.starts_with("tests/")),
        "tests/ must not be packaged: {files:?}"
    );
    // A template's build output, or a Monorepo template's Rust App's, when
    // it was built in place.
    ensure!(
        !files.iter().any(|file| {
            let parts: Vec<&str> = file.split('/').collect();
            parts.first() == Some(&"templates")
                && (parts.get(2) == Some(&"target")
                    || (parts.get(2) == Some(&"apps") && parts.get(4) == Some(&"target")))
        }),
        "a template's target/ must not be packaged: {files:?}"
    );
    Ok(())
}

/// The package ships exactly what `build.rs` embeds: every embedded
/// template file is packaged, and no packaged template file is left out of
/// the embedding — so neither a JS App's `node_modules` nor its build
/// output, installed or built in place, reaches the package. (Cargo leaves
/// out a directory holding a `Cargo.toml`: a template's manifest is
/// `Cargo.toml.template`.)
#[test]
fn package_ships_exactly_the_embedded_template_files() -> Result<()> {
    let embedded: BTreeSet<String> = rok_templates::EMBEDDED
        .iter()
        .flat_map(|template| {
            template
                .files
                .iter()
                .map(move |file| format!("templates/{}/{}", template.dir, file.path))
        })
        .collect();
    ensure!(!embedded.is_empty(), "templates/ holds no files");
    let packaged: BTreeSet<String> = packaged_files()?
        .into_iter()
        .filter(|file| file.starts_with("templates/"))
        .collect();
    let missing: Vec<_> = embedded.difference(&packaged).collect();
    ensure!(
        missing.is_empty(),
        "embedded template files not packaged: {missing:?}"
    );
    let stray: Vec<_> = packaged.difference(&embedded).collect();
    ensure!(
        stray.is_empty(),
        "packaged template files the embedding leaves out: {stray:?}"
    );
    Ok(())
}
