//! Every template's CI installs `rok` from the rok release its workflow
//! pins in `ROK_VERSION`, not by `cargo install rok-data-cli` (which builds
//! from source on every run, and installs no `rok`). The pin follows
//! `rok-data.pin`: a rok release's minor is the minor of the rok-data it
//! links (rok 0.7.x links rok-data 0.7), so the version of `rok` CI checks
//! the generated code with generates what the templates' rok-data does.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The crate's root, where `rok-data.pin` and `templates/` are.
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every GitHub Actions workflow of the templates: the `.yml` and `.yaml`
/// files of each `.github/workflows` directory, however deep (the
/// monorepo's API App has its own), in a stable order.
fn template_workflows() -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    collect_workflows(&root().join("templates"), &mut found)?;
    found.sort();
    Ok(found)
}

/// Adds the workflows under `dir` to `found`.
fn collect_workflows(dir: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        if path.is_dir() {
            collect_workflows(&path, found)?;
        } else if path
            .parent()
            .is_some_and(|p| p.ends_with(".github/workflows"))
            && path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            found.push(path);
        }
    }
    Ok(())
}

/// The value `workflow` gives `ROK_VERSION` (`ROK_VERSION: "0.7.0"`), if it
/// gives one.
fn rok_version(workflow: &str) -> Option<String> {
    workflow.lines().find_map(|line| {
        let value = line.trim().strip_prefix("ROK_VERSION:")?;
        Some(value.trim().trim_matches(['"', '\'']).to_owned())
    })
}

/// `major.minor` of `version` (`v0.7.0` or `0.7.0`).
fn major_minor(version: &str) -> Option<String> {
    let mut parts = version.trim().trim_start_matches('v').split('.');
    let (major, minor) = (parts.next()?, parts.next()?);
    let number = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    (number(major) && number(minor) && parts.next().is_some()).then(|| format!("{major}.{minor}"))
}

/// Whether `workflow` runs `rok` (a `run:` line or block line calling it).
fn runs_rok(workflow: &str) -> bool {
    workflow.lines().any(|line| {
        let line = line.trim();
        let line = line
            .strip_prefix("- run:")
            .or(line.strip_prefix("run:"))
            .unwrap_or(line);
        line.trim_start().starts_with("rok ")
    })
}

/// Each template workflow that runs `rok`, with its text, by path relative
/// to the crate.
fn rok_workflows() -> Result<Vec<(String, String)>> {
    let mut with_rok = Vec::new();
    for path in template_workflows()? {
        let text = fs::read_to_string(&path).with_context(|| path.display().to_string())?;
        let name = path
            .strip_prefix(root())
            .unwrap_or(&path)
            .display()
            .to_string()
            .replace('\\', "/");
        if runs_rok(&text) {
            with_rok.push((name, text));
        }
    }
    Ok(with_rok)
}

#[test]
fn every_template_workflow_pins_the_rok_release_of_the_rok_data_pin() -> Result<()> {
    let pin = fs::read_to_string(root().join("rok-data.pin")).context("rok-data.pin")?;
    let wanted = major_minor(&pin).with_context(|| format!("rok-data.pin: {pin:?}"))?;
    let found = rok_workflows()?;
    assert!(
        found
            .iter()
            .any(|(name, _)| name == "templates/axum-api/.github/workflows/ci.yml"),
        "no workflow running rok found: {:?}",
        found.iter().map(|(name, _)| name).collect::<Vec<_>>()
    );
    let mut versions = Vec::new();
    for (name, text) in &found {
        let version = rok_version(text).with_context(|| format!("{name} has no ROK_VERSION"))?;
        assert_eq!(
            major_minor(&version).as_deref(),
            Some(wanted.as_str()),
            "{name}: ROK_VERSION {version:?} does not link the rok-data {wanted} of rok-data.pin \
             ({}); pin a rok {wanted}.x release",
            pin.trim()
        );
        assert!(
            text.contains("rok-${ROK_VERSION}-x86_64-unknown-linux-musl")
                && text.contains("sha256sum -c"),
            "{name}: rok is not installed from the checksummed release"
        );
        versions.push((name.as_str(), version));
    }
    let first = &versions[0].1;
    assert!(
        versions.iter().all(|(_, version)| version == first),
        "the template workflows pin different rok releases: {versions:?}"
    );
    Ok(())
}

#[test]
fn no_template_workflow_builds_rok_from_source() -> Result<()> {
    let found = template_workflows()?;
    assert!(!found.is_empty(), "no template workflow found");
    for path in found {
        let text = fs::read_to_string(&path).with_context(|| path.display().to_string())?;
        assert!(
            !text.contains("cargo install rok-data-cli"),
            "{}: installs rok-data-cli from source; download the rok release instead",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn major_minor_reads_a_pin_and_a_release() {
    assert_eq!(major_minor("v0.7.0\n").as_deref(), Some("0.7"));
    assert_eq!(major_minor("0.8.1").as_deref(), Some("0.8"));
    assert_eq!(major_minor("0.8"), None);
    assert_eq!(major_minor("latest"), None);
}

/// GitHub runs a repository's workflows from the root's
/// `.github/workflows` only, so a monorepo App's workflow under its own
/// directory would never run.
#[test]
fn the_monorepos_workflows_sit_at_its_root() -> Result<()> {
    let monorepo = root().join("templates/monorepo");
    let at_root = monorepo.join(".github/workflows");
    let workflows: Vec<PathBuf> = template_workflows()?
        .into_iter()
        .filter(|path| path.starts_with(&monorepo))
        .collect();
    assert!(
        workflows.contains(&at_root.join("api.yml")),
        "the monorepo has no root .github/workflows/api.yml: {workflows:?}"
    );
    for path in &workflows {
        assert!(
            path.parent() == Some(at_root.as_path()),
            "{}: GitHub never runs a workflow outside the root's .github/workflows",
            path.display()
        );
    }
    Ok(())
}

/// Every template `Dockerfile`, however deep, by path relative to the
/// crate, with its text.
fn template_dockerfiles() -> Result<Vec<(String, String)>> {
    fn walk(dir: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir).with_context(|| dir.display().to_string())? {
            let path = entry?.path();
            if path.is_dir() {
                walk(&path, found)?;
            } else if path.file_name().is_some_and(|name| name == "Dockerfile") {
                found.push(path);
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    walk(&root().join("templates"), &mut paths)?;
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).with_context(|| path.display().to_string())?;
            let name = path
                .strip_prefix(root())
                .unwrap_or(&path)
                .display()
                .to_string()
                .replace('\\', "/");
            Ok((name, text))
        })
        .collect()
}

/// The image's migrate stage downloads the `rok` release CI pins, checked
/// against its SHA-256, instead of building an old `rok-data-cli`.
#[test]
fn every_template_dockerfile_downloads_the_rok_release_ci_pins() -> Result<()> {
    let workflows = rok_workflows()?;
    let pinned = rok_version(&workflows[0].1).context("the first workflow has no ROK_VERSION")?;
    let dockerfiles = template_dockerfiles()?;
    assert!(
        dockerfiles.len() >= 3,
        "expected the actix, axum and monorepo API Dockerfiles: {:?}",
        dockerfiles.iter().map(|(name, _)| name).collect::<Vec<_>>()
    );
    for (name, text) in &dockerfiles {
        assert!(
            !text.contains("cargo install rok-data-cli"),
            "{name}: builds rok-data-cli from source; download the rok release instead"
        );
        let version = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("ARG ROK_VERSION="))
            .with_context(|| format!("{name} has no `ARG ROK_VERSION=`"))?;
        assert_eq!(
            version, pinned,
            "{name}: ROK_VERSION {version} is not the {pinned} the workflows pin"
        );
        assert!(
            text.contains("rok-${ROK_VERSION}-${arch}-unknown-linux-musl")
                && text.contains("sha256sum -c"),
            "{name}: rok is not downloaded from the checksummed release"
        );
    }
    Ok(())
}
