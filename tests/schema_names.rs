//! Every type an API template derives `utoipa::ToSchema` for becomes an
//! `OpenAPI` component named after the type (or its `#[schema(as = ...)]`).
//! utoipa keeps one component per name: two different types under one name
//! leave every operation referring to whichever was registered last, so the
//! published contract describes a schema the operation does not answer
//! (the `UserResponse` of `users`, `account` and `auth`, 0027 review).
//!
//! `rok gen` refuses a project where two `ToSchema` types share a component
//! name, even identical copies (the `TokenResponse` of `auth`, `two_factor`
//! and `oidc`, starter-feedback ticket 96), so a freshly made project would
//! not generate. Features may not import each other, so a feature that
//! needs a small type `auth` also has keeps its own copy under its own name
//! (`TwoFactorTokenResponse`). This test fails when two types of one API
//! share a component name. A generic type is named per use, so it is left
//! out, as `rok gen` leaves it out.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The API templates (and the monorepo's api App), relative to
/// `templates/`, whose `src/` is scanned.
const APIS: &[&str] = &["axum-api", "actix-api", "monorepo/apps/api"];

/// The component names of the `ToSchema` types `text` (a Rust source
/// file) defines, generic types left out.
fn schemas_in(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let mut found = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if !lines[at].starts_with("#[derive(") {
            at += 1;
            continue;
        }
        let start = at;
        while at < lines.len() && !lines[at].ends_with(")]") {
            at += 1;
        }
        let derive = lines[start..=at.min(lines.len() - 1)].join(" ");
        at += 1;
        if !derive.contains("ToSchema") {
            continue;
        }
        // Attributes and doc comments up to the item.
        let mut renamed = None;
        while at < lines.len() && !is_item(lines[at]) {
            if let Some(name) = schema_as(lines[at]) {
                renamed = Some(name);
            }
            at += 1;
        }
        let Some(item) = lines.get(at) else { break };
        if !is_generic(item) {
            found.push(renamed.unwrap_or_else(|| item_name(item)));
        }
        at += 1;
    }
    found
}

/// Whether `line` starts a struct or an enum.
fn is_item(line: &str) -> bool {
    let line = line.strip_prefix("pub ").unwrap_or(line);
    let line = line.strip_prefix("pub(crate) ").unwrap_or(line);
    line.starts_with("struct ") || line.starts_with("enum ")
}

/// The word after `struct` or `enum` in `line`: the name and any generics.
fn item_word(line: &str) -> &str {
    line.split_whitespace()
        .skip_while(|word| *word != "struct" && *word != "enum")
        .nth(1)
        .unwrap_or_default()
}

/// Whether the struct or enum `line` starts takes generic parameters.
fn is_generic(line: &str) -> bool {
    item_word(line).contains('<')
}

/// The name of the struct or enum `line` starts, generics left out.
fn item_name(line: &str) -> String {
    item_word(line)
        .split(['<', '{', '(', ';'])
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// The component name a `#[schema(as = path::Name)]` attribute gives.
fn schema_as(line: &str) -> Option<String> {
    let rest = line.strip_prefix("#[schema(")?;
    let value = rest.split("as =").nth(1)?;
    let path = value.split([',', ')']).next()?.trim();
    Some(path.rsplit("::").next().unwrap_or(path).to_owned())
}

/// Every `.rs` file under `dir`, by its path relative to `root`.
fn rust_files(root: &Path, dir: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files(root, &path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let relative = path.strip_prefix(root)?.display().to_string();
            let text =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            files.insert(relative.replace('\\', "/"), text.replace("\r\n", "\n"));
        }
    }
    Ok(())
}

/// Each component name more than one type shares among `files` (path →
/// source), as a line naming the name and the files.
fn collisions(files: &BTreeMap<String, String>) -> Vec<String> {
    let mut by_name: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for (path, text) in files {
        for name in schemas_in(text) {
            by_name.entry(name).or_default().push(path);
        }
    }
    by_name
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(name, paths)| format!("`{name}`: {}", paths.join(", ")))
        .collect()
}

/// The templates of this crate.
fn templates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

/// `rok gen` refuses a project whose `ToSchema` types share a component
/// name, so every API template must give each its own.
#[test]
fn no_two_schemas_of_an_api_template_share_a_component_name() -> Result<()> {
    for api in APIS {
        let src = templates().join(api).join("src");
        let mut files = BTreeMap::new();
        rust_files(&src, &src, &mut files)?;
        let found = collisions(&files);
        assert!(
            found.is_empty(),
            "templates/{api}: `ToSchema` types share a component name, so `rok gen` fails \
             on a fresh project (rename one, or `#[schema(as = ...)]`):\n  {}",
            found.join("\n  ")
        );
    }
    Ok(())
}

/// The scanner reads the component names a derive and a renaming attribute
/// make, leaves generic types out, and names every shared name, identical
/// copies included.
#[test]
fn the_scanner_names_every_shared_component_name() {
    let users = "/// A row.\n#[derive(\n    Debug, utoipa::ToSchema,\n)]\npub struct UserResponse {\n    pub id: i64,\n}\n";
    let account = "#[derive(Debug, utoipa::ToSchema)]\n#[schema(as = crate::Profile)]\npub struct UserResponse {\n    pub id: i64,\n}\n\n#[derive(utoipa::ToSchema)]\npub struct Page<T> {\n    pub items: Vec<T>,\n}\n#[derive(Debug)]\npub struct Plain;\n";
    assert_eq!(schemas_in(account), ["Profile"]);

    let files = BTreeMap::from([
        ("users.rs".to_owned(), users.to_owned()),
        ("account.rs".to_owned(), account.to_owned()),
        ("pages.rs".to_owned(), account.replace("Profile", "Other")),
    ]);
    assert!(collisions(&files).is_empty());
    let mut files = files;
    files.insert("copy.rs".to_owned(), users.to_owned());
    assert_eq!(collisions(&files), ["`UserResponse`: copy.rs, users.rs"]);
}
