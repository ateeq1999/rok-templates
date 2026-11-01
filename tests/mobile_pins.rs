//! The monorepo's mobile App pins every dependency to an exact version
//! (workspace ADR 0027: no ranges, no `latest`), the version its checked-in
//! `pubspec.lock` resolves, so `flutter pub get --enforce-lockfile` (the
//! mobile gate's) installs exactly what the template was tested with.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// The mobile App of the monorepo template.
fn mobile() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/monorepo/apps/mobile")
}

/// The dependencies `pubspec` (a `pubspec.yaml`) gives a version on their
/// own line, under `dependencies:` and `dev_dependencies:`, by name; those
/// given by `sdk:` or `path:` are left out.
fn constraints(pubspec: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    let mut in_dependencies = false;
    for line in pubspec.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() {
            in_dependencies = matches!(line.trim(), "dependencies:" | "dev_dependencies:");
            continue;
        }
        if !in_dependencies || line.starts_with("    ") {
            continue;
        }
        let line = line.split(" #").next().unwrap_or(line).trim();
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim().trim_matches('"');
            if !value.is_empty() {
                found.insert(name.trim().to_owned(), value.to_owned());
            }
        }
    }
    found
}

/// The versions `lock` (a `pubspec.lock`) resolves, by package.
fn locked(lock: &str) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    let mut package = None;
    for line in lock.lines() {
        if let Some(name) = line.strip_prefix("  ").and_then(|l| l.strip_suffix(':'))
            && !name.starts_with(' ')
        {
            package = Some(name.to_owned());
        } else if let (Some(name), Some(version)) =
            (&package, line.trim().strip_prefix("version: "))
        {
            found.insert(name.clone(), version.trim_matches('"').to_owned());
        }
    }
    found
}

/// Whether `value` is one exact version (`1.2.3`, `10.3.0+2`), not a range.
fn is_exact(value: &str) -> bool {
    let core = value.split(['+', '-']).next().unwrap_or(value);
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

#[test]
fn every_mobile_dependency_is_pinned_to_the_version_its_lock_resolves() -> Result<()> {
    let pubspec = fs::read_to_string(mobile().join("pubspec.yaml")).context("pubspec.yaml")?;
    let lock = fs::read_to_string(mobile().join("pubspec.lock")).context("pubspec.lock")?;
    let (wanted, resolved) = (constraints(&pubspec), locked(&lock));
    assert!(wanted.contains_key("flutter_bloc"), "{wanted:?}");
    let wrong: Vec<String> = wanted
        .iter()
        .filter(|(name, value)| !is_exact(value) || resolved.get(*name) != Some(*value))
        .map(|(name, value)| {
            format!(
                "{name}: {value} (pubspec.lock resolves {})",
                resolved.get(name).map_or("nothing", String::as_str)
            )
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "apps/mobile/pubspec.yaml must pin each dependency to the exact version \
         pubspec.lock resolves:\n  {}",
        wrong.join("\n  ")
    );
    Ok(())
}

#[test]
fn the_readers_tell_a_pin_from_a_range() {
    let pubspec = "name: m\nenvironment:\n  sdk: ^3.0.0\ndependencies:\n  flutter:\n    sdk: flutter\n  a: 1.2.3\n  b: ^1.0.0 # why\n  c: \"10.3.0+2\"\ndev_dependencies:\n  d: any\n";
    let found = constraints(pubspec);
    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        [
            ("a".to_owned(), "1.2.3".to_owned()),
            ("b".to_owned(), "^1.0.0".to_owned()),
            ("c".to_owned(), "10.3.0+2".to_owned()),
            ("d".to_owned(), "any".to_owned()),
        ]
    );
    assert!(is_exact("1.2.3") && is_exact("10.3.0+2"));
    assert!(!is_exact("^1.0.0") && !is_exact("any") && !is_exact(">=1.0.0 <2.0.0"));
    let lock = "packages:\n  a:\n    dependency: \"direct main\"\n    description:\n      name: a\n    source: hosted\n    version: \"1.2.3\"\nsdks:\n  dart: \">=3.0.0 <4.0.0\"\n";
    assert_eq!(locked(lock).get("a").map(String::as_str), Some("1.2.3"));
}
