//! What `cargo package` ships: every file of every template, so
//! `init --template` from `cargo install` copies what it does from the
//! repository, and no template's build output.

use std::fs;
use std::path::{Path, PathBuf};
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
    // A template's build output, when the template was built in place.
    ensure!(
        !files
            .iter()
            .any(|file| file.starts_with("templates/") && file.split('/').nth(2) == Some("target")),
        "a template's target/ must not be packaged: {files:?}"
    );
    Ok(())
}

/// Every file of every template is packaged. (Cargo leaves out a directory
/// holding a `Cargo.toml`: a template's manifest is `Cargo.toml.template`.)
#[test]
fn package_ships_every_template_file() -> Result<()> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                // A template's build in place.
                if relative.split('/').nth(2) == Some("target") {
                    continue;
                }
                walk(&path, root, out)?;
            } else {
                out.push(relative);
            }
        }
        Ok(())
    }
    let mut templates = Vec::new();
    walk(&root().join("templates"), &root(), &mut templates)?;
    ensure!(!templates.is_empty(), "templates/ holds no files");
    let files = packaged_files()?;
    let missing: Vec<_> = templates
        .iter()
        .filter(|file| !files.contains(file))
        .collect();
    ensure!(
        missing.is_empty(),
        "template files not packaged: {missing:?}"
    );
    Ok(())
}
