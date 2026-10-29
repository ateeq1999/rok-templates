//! The project templates `rok sqlc init --template` copies: each directory
//! of `templates/`, embedded in the crate by `build.rs` so `init` works
//! from `cargo install`.
//!
//! This crate holds the templates' files and nothing else. Which name
//! selects which template, and which files a project gets, is the engine's
//! (`rok-data-sqlc`'s `init`), because `rok sqlc init` is its grammar
//! (workspace ADR 0022).

/// A template file, as `build.rs` embeds it.
#[derive(Debug)]
pub struct File {
    /// Its path in the template, with `/`.
    pub path: &'static str,
    /// Its contents.
    pub bytes: &'static [u8],
}

/// A template directory of `templates/` and its files, as `build.rs`
/// embeds it.
#[derive(Debug)]
pub struct Embedded {
    /// The directory's name, such as `axum-api`.
    pub dir: &'static str,
    /// Every file but those of the directory's `target/`, sorted by path.
    pub files: &'static [File],
}

/// Every template directory of `templates/`, sorted by name.
pub const EMBEDDED: &[Embedded] = include!(concat!(env!("OUT_DIR"), "/templates.rs"));

/// The template directory `dir`, if there is one.
#[must_use]
pub fn find(dir: &str) -> Option<&'static Embedded> {
    EMBEDDED.iter().find(|embedded| embedded.dir == dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_templates_are_embedded_with_their_manifest() {
        for dir in ["actix-api", "axum-api"] {
            let embedded = find(dir).unwrap_or_else(|| panic!("`{dir}` is not embedded"));
            assert!(
                embedded
                    .files
                    .iter()
                    .any(|file| file.path == "Cargo.toml.template"),
                "`{dir}` has no Cargo.toml.template"
            );
        }
    }

    #[test]
    fn no_build_output_is_embedded() {
        for embedded in EMBEDDED {
            assert!(
                !embedded
                    .files
                    .iter()
                    .any(|file| file.path.starts_with("target/")),
                "`{}` embeds its target/",
                embedded.dir
            );
        }
    }
}
