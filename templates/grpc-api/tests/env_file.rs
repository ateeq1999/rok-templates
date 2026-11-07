//! `.env`: a debug build reads the first one in the directory or above it,
//! without overriding what is set, and a value may name an earlier one
//! (`${POSTGRES_PORT}`); a release build reads none.

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
