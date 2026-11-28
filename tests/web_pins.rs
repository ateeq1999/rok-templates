//! The monorepo's web App regenerates its route tree with the router's own
//! generator, runs its tests on any Node, and is gated on one pinned Node.
//!
//! - `@tanstack/router-cli` (the `tsr` of `bun run routes`) is pinned to the
//!   version of the `@tanstack/router-generator` that `@tanstack/router-plugin`
//!   resolves to in `bun.lock`: the generator `vite dev` runs, so the two
//!   write the same `routeTree.gen.ts`.
//! - `bun run test` turns Node's own Web Storage off (Node 25+ puts an
//!   unusable `localStorage` over jsdom's), so no Node needs a flag added.
//! - CI's web gate runs one exact Node release.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// The web App of the monorepo template.
fn web() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/monorepo/apps/web")
}

/// The value of the first `"<key>": "<value>"` in `json` after `from`.
fn string_after<'a>(json: &'a str, from: &str, key: &str) -> Option<&'a str> {
    let rest = &json[json.find(from)?..];
    let rest = &rest[rest.find(&format!("\"{key}\": \""))? + key.len() + 5..];
    rest.split('"').next()
}

/// The version `package.json` asks for `package` at.
fn wanted(package_json: &str, package: &str) -> Option<String> {
    string_after(package_json, "\"devDependencies\"", package)
        .or_else(|| string_after(package_json, "\"dependencies\"", package))
        .map(str::to_owned)
}

/// The version `bun.lock` resolves `dependency` to for `package`.
fn locked_dependency(lock: &str, package: &str, dependency: &str) -> Option<String> {
    string_after(lock, &format!("\"{package}\": [\"{package}@"), dependency).map(str::to_owned)
}

#[test]
fn the_route_cli_is_the_router_plugins_generator() -> Result<()> {
    let package = fs::read_to_string(web().join("package.json")).context("package.json")?;
    let lock = fs::read_to_string(web().join("bun.lock")).context("bun.lock")?;
    let generator = locked_dependency(
        &lock,
        "@tanstack/router-plugin",
        "@tanstack/router-generator",
    )
    .context("bun.lock resolves no router generator for @tanstack/router-plugin")?;
    assert_eq!(
        wanted(&package, "@tanstack/router-cli").as_deref(),
        Some(generator.as_str()),
        "apps/web/package.json must pin @tanstack/router-cli to {generator}, the \
         @tanstack/router-generator its @tanstack/router-plugin runs"
    );
    assert_eq!(
        string_after(&package, "\"scripts\"", "routes"),
        Some("tsr generate"),
        "apps/web/package.json's `routes` script regenerates the route tree"
    );
    Ok(())
}

#[test]
fn the_test_script_turns_nodes_web_storage_off() -> Result<()> {
    let package = fs::read_to_string(web().join("package.json")).context("package.json")?;
    assert_eq!(
        string_after(&package, "\"scripts\"", "test"),
        Some("NODE_OPTIONS=--no-experimental-webstorage vitest run"),
    );
    Ok(())
}

#[test]
fn the_web_gate_runs_one_exact_node() -> Result<()> {
    let ci = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml"),
    )
    .context("ci.yml")?;
    let gate = &ci[ci
        .find("if: matrix.gate == 'web'\n        with:\n          node-version:")
        .context("ci.yml's web gate sets up no Node")?..];
    let version = gate
        .lines()
        .find_map(|line| line.trim().strip_prefix("node-version: "))
        .unwrap_or_default();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() == 3 && parts.iter().all(|p| p.parse::<u32>().is_ok()),
        "ci.yml's web gate must pin Node to one release (x.y.z), not `{version}`"
    );
    Ok(())
}

#[test]
fn the_readers_find_versions() {
    let package = "{\n  \"scripts\": {\n    \"test\": \"vitest run\"\n  },\n  \"dependencies\": {\n    \"a\": \"1.0.0\"\n  },\n  \"devDependencies\": {\n    \"b\": \"2.0.0\"\n  }\n}";
    assert_eq!(wanted(package, "a").as_deref(), Some("1.0.0"));
    assert_eq!(wanted(package, "b").as_deref(), Some("2.0.0"));
    assert_eq!(wanted(package, "c"), None);
    let lock = "    \"p\": [\"p@1.0.0\", \"\", { \"dependencies\": { \"g\": \"1.2.3\" } }],";
    assert_eq!(locked_dependency(lock, "p", "g").as_deref(), Some("1.2.3"));
}
