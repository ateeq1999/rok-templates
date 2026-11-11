//! The two API templates share their framework-independent code (engine
//! ADR 0021, A6): `templates/actix-api` and `templates/axum-api` hold byte-identical
//! copies of the files [`SHARED`] lists. Each template stays
//! self-contained, so the copies are checked here: an edit made to one
//! template only fails this test, naming the file.
//!
//! To share more, add a path to [`SHARED`]: a directory (every file under
//! it, on both sides) or a file. A feature listed in [`FEATURES`] shares
//! its [`FEATURE_FILES`] that exist in either template; a hand-written
//! feature listed in [`HAND_WRITTEN`], its [`HAND_WRITTEN_FILES`].

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The directories and files both templates hold the same copy of.
const SHARED: &[&str] = &[
    "migrations",
    // The truth `rok gen --migration` stages `migrations/` from.
    "schema",
    "queries",
    "src/shared/security",
    // All of `shared/mail`: the sender is started (`tokio::spawn` or
    // actix's runtime) and stopped by each template's `app.rs`.
    "src/shared/mail",
    // The rate limit's counting and `client_ip`; each `rate_limit/mod.rs`
    // is the framework's middleware around it.
    "src/shared/rate_limit/limiter.rs",
    // The `Storage` port and its `object_store` implementation.
    "src/shared/storage",
    // The profile photos' image processing: decode, crop, re-encode.
    "src/shared/thumbnail.rs",
    // The ports features declare for each other, implemented: the wiring's
    // one framework-free part.
    "src/app/ports.rs",
    // The development seed (`cargo run --bin seed`): the database alone.
    "src/app/seed.rs",
    // Logs and traces: the subscriber, `traceparent`, the caller on the span.
    "src/shared/telemetry.rs",
    // The metrics: their names, the exporter, the pool sampler.
    "src/shared/metrics.rs",
    // The settings readers and their errors.
    "src/shared/settings.rs",
    // The drain on `SIGTERM`: the flag, its settings, the signal.
    "src/shared/shutdown.rs",
    // Running and shipping it: the image, the compose stack, CI.
    "Dockerfile",
    ".dockerignore",
    "compose.yaml",
    "deploy",
    ".github",
    // The job queue: `Job`, `Queue`, `Worker`.
    "src/shared/jobs",
    // The live Topics: `publish`, the hub and its tickets.
    "src/shared/live",
    "src/shared/mod.rs",
    // `cargo deny check`'s policy.
    "deny.toml",
];

/// The features whose framework-independent files both templates share.
const FEATURES: &[&str] = &["users", "audit_log_entry"];

/// The files of a feature no framework shows through: only `routes.rs`
/// and `handlers.rs` (and `mod.rs`, which names them) are rewritten.
/// A generated feature's scaffolded `service.rs` is left out: here it
/// names the framework's wiring (`register_services` or `router`).
const FEATURE_FILES: &[&str] = &["dto.rs", "model.rs", "repo.rs", "service_api.rs"];

/// The hand-written features (the account flows, API keys and uploads) whose
/// [`HAND_WRITTEN_FILES`] both templates share: they have no `model.rs`,
/// `repo.rs` or `service_api.rs`.
const HAND_WRITTEN: &[&str] = &[
    "auth",
    "account",
    "api_keys",
    "jobs",
    "live",
    "media",
    "two_factor",
];

/// The files of a hand-written feature no framework shows through.
const HAND_WRITTEN_FILES: &[&str] = &["dto.rs", "service.rs"];

/// The paths, relative to a template, that [`SHARED`], [`FEATURES`] and
/// [`FEATURE_FILES`] (and [`HAND_WRITTEN`] and
/// [`HAND_WRITTEN_FILES`]) name.
fn shared_paths() -> Vec<String> {
    let mut paths: Vec<String> = SHARED.iter().map(|path| (*path).to_owned()).collect();
    for feature in FEATURES {
        for file in FEATURE_FILES {
            paths.push(format!("src/features/{feature}/{file}"));
        }
    }
    for feature in HAND_WRITTEN {
        for file in HAND_WRITTEN_FILES {
            paths.push(format!("src/features/{feature}/{file}"));
        }
    }
    paths
}

/// Every file under `root/path` (or `root/path` itself, a file), by its
/// path relative to `root` with `/`, and its text with `\n` line endings.
/// None when it doesn't exist.
fn files_under(root: &Path, path: &str) -> Result<BTreeMap<String, String>> {
    fn walk(root: &Path, at: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
        if at.is_dir() {
            for entry in fs::read_dir(at).with_context(|| format!("reading {}", at.display()))? {
                walk(root, &entry?.path(), files)?;
            }
        } else if at.is_file() {
            let relative = at
                .strip_prefix(root)?
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let text =
                fs::read_to_string(at).with_context(|| format!("reading {}", at.display()))?;
            files.insert(relative, text.replace("\r\n", "\n"));
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    walk(root, &root.join(path), &mut files)?;
    Ok(files)
}

/// Each shared file that differs between the templates at `left` and
/// `right`, or that only one of them has, as a line naming it.
fn drift(left: &Path, right: &Path, paths: &[String]) -> Result<Vec<String>> {
    let (left_name, right_name) = (dir_name(left), dir_name(right));
    let mut found = Vec::new();
    for path in paths {
        let (ours, theirs) = (files_under(left, path)?, files_under(right, path)?);
        let names: std::collections::BTreeSet<&String> = ours.keys().chain(theirs.keys()).collect();
        for name in names {
            match (ours.get(name), theirs.get(name)) {
                (Some(a), Some(b)) if a != b => found.push(format!("{name}: differs")),
                (Some(_), None) => found.push(format!("{name}: only in {left_name}")),
                (None, Some(_)) => found.push(format!("{name}: only in {right_name}")),
                _ => {}
            }
        }
    }
    Ok(found)
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The templates of this crate.
fn templates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates")
}

#[test]
fn the_templates_share_their_framework_independent_files() -> Result<()> {
    let (actix, axum) = (templates().join("actix-api"), templates().join("axum-api"));
    for path in shared_paths() {
        let exists = actix.join(&path).exists() || axum.join(&path).exists();
        assert!(
            exists,
            "`{path}` is listed as shared but neither template has it"
        );
    }
    let found = drift(&actix, &axum, &shared_paths())?;
    assert!(
        found.is_empty(),
        "the templates' shared files drifted apart (make both copies the same):\n  {}",
        found.join("\n  ")
    );
    Ok(())
}

/// The files `templates/grpc-api` holds the same copy of as `templates/axum-api`
/// (workspace ADR 0030): the configuration, the data layer's sources and its
/// framework-free generated files, the jobs, mail, storage, telemetry,
/// metrics, shutdown and security, the development seed, and the
/// generated features' files no transport shows through. What differs is
/// the transport's: the wiring, the error's answer, the caller, and the
/// generated files that derive
/// the OpenAPI schemas (`db/*/models.rs`, `dto.rs`), which gRPC has none of.
const GRPC_SHARED: &[&str] = &[
    "migrations",
    "schema",
    "queries",
    "src/config.rs",
    // The data layer's files no framework shows through.
    "src/db/queries",
    "src/db/pool.rs",
    "src/db/nullable.rs",
    "src/db/schema.rs",
    "src/shared/jobs",
    "src/shared/mail",
    "src/shared/storage",
    "src/shared/telemetry.rs",
    "src/shared/metrics.rs",
    "src/shared/settings.rs",
    "src/shared/shutdown.rs",
    "src/shared/security",
    "src/app/seed.rs",
];

/// The files of a generated feature both the axum and the gRPC template
/// hold: the scaffolded `service.rs` names no framework's wiring there, nor
/// does the generated `service_api.rs`.
const GRPC_FEATURE_FILES: &[&str] = &["model.rs", "repo.rs", "service.rs", "service_api.rs"];

/// The paths, relative to a template, that [`GRPC_SHARED`] and the
/// [`FEATURES`]' [`GRPC_FEATURE_FILES`] name.
fn grpc_shared_paths() -> Vec<String> {
    let mut paths: Vec<String> = GRPC_SHARED.iter().map(|path| (*path).to_owned()).collect();
    for feature in FEATURES {
        for file in GRPC_FEATURE_FILES {
            paths.push(format!("src/features/{feature}/{file}"));
        }
    }
    paths
}

#[test]
fn the_grpc_template_shares_the_axum_templates_files() -> Result<()> {
    let (axum, grpc) = (templates().join("axum-api"), templates().join("grpc-api"));
    for path in grpc_shared_paths() {
        assert!(
            axum.join(&path).exists() && grpc.join(&path).exists(),
            "`{path}` is listed as shared with grpc-api but a template lacks it"
        );
    }
    let found = drift(&axum, &grpc, &grpc_shared_paths())?;
    assert!(
        found.is_empty(),
        "grpc-api's files shared with axum-api drifted apart (make both copies the same):\n  {}",
        found.join("\n  ")
    );
    Ok(())
}

/// The axum template's files the monorepo's api App holds its own way,
/// because the monorepo builds and checks it from the project root:
/// GitHub runs workflows from the root's `.github/workflows` only (the
/// App's is the root's `api.yml`), and the image builds from the root,
/// where the workspace's `Cargo.lock` is (its ignore list is
/// `Dockerfile.dockerignore`, read for `-f apps/api/Dockerfile`).
const BUILT_FROM_THE_ROOT: &[&str] = &[".github/workflows/ci.yml", ".dockerignore", "Dockerfile"];

/// The monorepo's api App is the axum template file for file (workspace
/// ADR 0027): every file of `templates/axum-api` but its manifest and lock
/// file (the monorepo's root holds the lock file, and init renames the
/// App's package) has the same copy under `templates/monorepo/apps/api`,
/// which adds only its own files (the contract's export and the tests
/// that drive the API through the Rust client). [`BUILT_FROM_THE_ROOT`]
/// are the axum template's files the App holds its own way.
#[test]
fn the_monorepo_api_app_is_the_axum_template() -> Result<()> {
    let (axum, api) = (
        templates().join("axum-api"),
        templates().join("monorepo/apps/api"),
    );
    let paths: Vec<String> = files_under(&axum, "")?
        .into_keys()
        .filter(|path| {
            !path.starts_with("target/")
                && path != "Cargo.lock"
                && path != "Cargo.toml.template"
                && !BUILT_FROM_THE_ROOT.contains(&path.as_str())
        })
        .collect();
    let found = drift(&axum, &api, &paths)?;
    assert!(
        found.is_empty(),
        "the monorepo's apps/api drifted from the axum template (make both copies the same):\n  {}",
        found.join("\n  ")
    );
    Ok(())
}

/// The comparer names a changed file, a file one side lacks, and a file
/// that differs only in line endings as no difference.
#[test]
fn the_comparer_names_each_injected_difference() -> Result<()> {
    let scratch = tempfile::TempDir::new()?;
    let (left, right) = (scratch.path().join("left"), scratch.path().join("right"));
    for root in [&left, &right] {
        fs::create_dir_all(root.join("queries"))?;
        fs::create_dir_all(root.join("src/features/users"))?;
        fs::write(root.join("queries/mail.sql"), "SELECT 1;\n")?;
        fs::write(root.join("src/features/users/dto.rs"), "pub struct Dto;\n")?;
    }
    fs::write(right.join("queries/mail.sql"), "SELECT 1;\r\n")?;
    let paths = ["queries".to_owned(), "src/features/users/dto.rs".to_owned()];
    assert_eq!(drift(&left, &right, &paths)?, Vec::<String>::new());

    fs::write(
        right.join("src/features/users/dto.rs"),
        "pub struct Other;\n",
    )?;
    fs::write(left.join("queries/session.sql"), "SELECT 2;\n")?;
    assert_eq!(
        drift(&left, &right, &paths)?,
        [
            "queries/session.sql: only in left",
            "src/features/users/dto.rs: differs",
        ]
    );
    Ok(())
}
