//! Every type an API template derives `utoipa::ToSchema` for becomes an
//! `OpenAPI` component named after the type (or its `#[schema(as = ...)]`).
//! utoipa keeps one component per name: two different types under one name
//! leave every operation referring to whichever was registered last, so the
//! published contract describes a schema the operation does not answer
//! (the `UserResponse` of `users`, `account` and `auth`, 0027 review).
//!
//! Features may not import each other, so a hand-written feature sometimes
//! keeps its own copy of a small type (`MessageResponse` in `auth` and
//! `two_factor`): identical copies make the same component and are fine.
//! This test fails when one name is given to definitions that differ.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The API templates (and the monorepo's api App), relative to
/// `templates/`, whose `src/` is scanned.
const APIS: &[&str] = &["axum-api", "actix-api", "monorepo/apps/api"];

/// One `ToSchema` type: the component name it makes and its definition
/// (derive, attributes and item), each line trimmed.
#[derive(Debug, PartialEq, Eq)]
struct Schema {
    name: String,
    definition: String,
}

/// The `ToSchema` types `text` (a Rust source file) defines.
fn schemas_in(text: &str) -> Vec<Schema> {
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
        let name = renamed.unwrap_or_else(|| item_name(item));
        if item.ends_with('{') {
            while at < lines.len() && lines[at] != "}" {
                at += 1;
            }
        }
        let end = at.min(lines.len() - 1);
        found.push(Schema {
            name,
            definition: lines[start..=end].join("\n"),
        });
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

/// The name of the struct or enum `line` starts, generics left out.
fn item_name(line: &str) -> String {
    line.split_whitespace()
        .skip_while(|word| *word != "struct" && *word != "enum")
        .nth(1)
        .unwrap_or_default()
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

/// Each component name that differing definitions share among `files`
/// (path → source), as a line naming the name and the files.
fn collisions(files: &BTreeMap<String, String>) -> Vec<String> {
    let mut by_name: BTreeMap<String, Vec<(&str, String)>> = BTreeMap::new();
    for (path, text) in files {
        for schema in schemas_in(text) {
            by_name
                .entry(schema.name)
                .or_default()
                .push((path, schema.definition));
        }
    }
    by_name
        .into_iter()
        .filter(|(_, definitions)| definitions.iter().any(|(_, d)| *d != definitions[0].1))
        .map(|(name, definitions)| {
            let paths: Vec<&str> = definitions.iter().map(|(path, _)| *path).collect();
            format!("`{name}`: {}", paths.join(", "))
        })
        .collect()
}

/// The templates of this crate.
fn templates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

#[test]
fn no_two_different_schemas_of_an_api_template_share_a_component_name() -> Result<()> {
    for api in APIS {
        let src = templates().join(api).join("src");
        let mut files = BTreeMap::new();
        rust_files(&src, &src, &mut files)?;
        let found = collisions(&files);
        assert!(
            found.is_empty(),
            "templates/{api}: different `ToSchema` types share a component name, so the \
             contract describes only one of them (rename one, or `#[schema(as = ...)]`):\n  {}",
            found.join("\n  ")
        );
    }
    Ok(())
}

/// The scanner reads the component names a derive, a renaming attribute
/// and a generic make, and names a collision but not identical copies.
#[test]
fn the_scanner_names_differing_definitions_but_not_identical_copies() {
    let users = "/// A row.\n#[derive(\n    Debug, utoipa::ToSchema,\n)]\npub struct UserResponse {\n    pub id: i64,\n}\n";
    let account = "#[derive(Debug, utoipa::ToSchema)]\n#[schema(as = crate::Profile)]\npub struct UserResponse {\n    pub id: i64,\n}\n\n#[derive(utoipa::ToSchema)]\npub struct Page<T> {\n    pub items: Vec<T>,\n}\n#[derive(Debug)]\npub struct Plain;\n";
    let names: Vec<String> = schemas_in(account).into_iter().map(|s| s.name).collect();
    assert_eq!(names, ["Profile", "Page"]);

    let auth = "#[derive(Debug, utoipa::ToSchema)]\npub struct UserResponse {\n    pub email: String,\n}\n";
    let files = BTreeMap::from([
        ("users.rs".to_owned(), users.to_owned()),
        ("copy.rs".to_owned(), users.to_owned()),
        ("account.rs".to_owned(), account.to_owned()),
    ]);
    assert!(collisions(&files).is_empty());
    let mut files = files;
    files.insert("auth.rs".to_owned(), auth.to_owned());
    assert_eq!(
        collisions(&files),
        ["`UserResponse`: auth.rs, copy.rs, users.rs"]
    );
}
