//! `.env`: a debug build reads the first one in the directory or above it,
//! without overriding what is set, and a value may name an earlier one
//! (`${POSTGRES_PORT}`); a release build reads none. The database tests
//! read their `DATABASE_URL` after it, and skip without one, except in CI.

#[path = "support/mod.rs"]
mod support;

use std::fs;

use grpc_api::config::load_env_file_from;

#[test]
fn a_debug_build_reads_the_env_file_and_a_release_build_does_not()
-> Result<(), Box<dyn std::error::Error>> {
    let suffix = uuid::Uuid::now_v7().simple().to_string().to_uppercase();
    let root = std::env::temp_dir().join(format!("env-file-{suffix}"));
    let nested = root.join("apps/api");
    fs::create_dir_all(&nested)?;
    // Names no other test reads, so setting them races with nothing.
    let (port, url) = (format!("PORT_{suffix}"), format!("URL_{suffix}"));
    let file = format!(
        "{port}=55432\n{url}=postgres://app:app@localhost:${{{port}}}/app\n\
         CARGO_MANIFEST_DIR=elsewhere\n"
    );
    fs::write(root.join(".env"), file)?;

    let read = load_env_file_from(&nested);
    fs::remove_dir_all(&root)?;
    if cfg!(debug_assertions) {
        assert_eq!(read?, Some(root.join(".env")));
        assert_eq!(
            std::env::var(&url)?,
            "postgres://app:app@localhost:55432/app"
        );
    } else {
        assert_eq!(read?, None);
        assert!(std::env::var(&url).is_err());
    }
    // What the environment sets, the file doesn't override.
    assert_eq!(
        std::env::var("CARGO_MANIFEST_DIR")?,
        env!("CARGO_MANIFEST_DIR")
    );
    Ok(())
}

#[test]
fn the_database_tests_read_their_url_and_skip_without_one() {
    let url = |name: &str| (name == "DATABASE_URL").then(|| "postgres://db".to_owned());
    assert_eq!(support::database_url(url).as_deref(), Some("postgres://db"));
    assert_eq!(support::database_url(|_| None), None);
}

#[test]
#[should_panic(expected = "DATABASE_URL is required in CI")]
fn in_ci_the_database_tests_fail_without_a_url() {
    support::database_url(|name| (name == "CI").then(|| "true".to_owned()));
}
