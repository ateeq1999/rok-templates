//! The monorepo template's root runs every App from one place (workspace
//! ADR 0027), whatever the shell's current directory:
//!
//! - a scripts-only `package.json`: `dev:web`, `dev:api` and `test:web`,
//!   with no workspaces and no dependencies, so the web App stays a plain
//!   bun project inside `apps/web`;
//! - `dev:api` names the API's package (`cargo run -p axum-api`, renamed
//!   after the project at init);
//! - the root `Cargo.toml`'s `default-members` is the API alone, so a bare
//!   `cargo run` at the root starts it, while `members` keeps every Rust
//!   App and the Rust Client for `cargo test --workspace`.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// The monorepo template's root.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/monorepo")
}

/// The value of `"<key>": "<value>"` in `json`, if it has one.
fn string_value<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\": \"");
    let rest = &json[json.find(&needle)? + needle.len()..];
    rest.split('"').next()
}

#[test]
fn the_root_package_json_runs_the_apps_from_the_root() -> Result<()> {
    let package = fs::read_to_string(root().join("package.json")).context("package.json")?;
    assert_eq!(
        string_value(&package, "dev:web"),
        Some("bun run --cwd apps/web dev"),
        "`dev:web` starts the web App's dev server from the root"
    );
    assert_eq!(
        string_value(&package, "test:web"),
        Some("bun run --cwd apps/web test"),
        "`test:web` runs the web App's tests from the root"
    );
    assert_eq!(
        string_value(&package, "dev:api"),
        Some("cargo run -p axum-api"),
        "`dev:api` names the API's package"
    );
    Ok(())
}

#[test]
fn the_root_package_json_is_scripts_only() -> Result<()> {
    let package = fs::read_to_string(root().join("package.json")).context("package.json")?;
    for key in [
        "workspaces",
        "dependencies",
        "devDependencies",
        "packageManager",
    ] {
        assert!(
            !package.contains(&format!("\"{key}\"")),
            "the root package.json has `{key}`: it holds scripts only, and the web App \
             stays a plain bun project inside apps/web (workspace ADR 0027)"
        );
    }
    Ok(())
}

#[test]
fn a_bare_cargo_run_at_the_root_starts_the_api() -> Result<()> {
    let manifest: toml::Table = fs::read_to_string(root().join("Cargo.toml.template"))
        .context("Cargo.toml.template")?
        .parse()?;
    let workspace = manifest["workspace"].as_table().context("no [workspace]")?;
    assert_eq!(
        workspace.get("default-members"),
        Some(&toml::Value::Array(vec!["apps/api".into()])),
        "the root workspace's default member is the API"
    );
    assert_eq!(
        workspace.get("members"),
        Some(&toml::Value::Array(vec![
            "apps/api".into(),
            "apps/desktop".into(),
            "clients/rust".into(),
        ])),
        "every Rust App and the Rust Client stay members, for `cargo test --workspace`"
    );
    Ok(())
}
