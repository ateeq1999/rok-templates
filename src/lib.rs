//! The project templates `rok sqlc init --template` copies: each directory
//! of `templates/`, embedded in the crate by `build.rs` so `init` works
//! from `cargo install`.
//!
//! The [`catalog`] owns the template names, their manifests and the
//! destination paths a project gets their files at (workspace ADR 0027).
//! Which name the grammar takes, and the write policy — copy when absent,
//! skip files marked `@generated` — stay the engine's (`rok-data-sqlc`'s
//! `init`), so the marker rules stay with the engine.

pub mod catalog;

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

    /// Neither a template's `target/` nor a Monorepo template's Rust App's
    /// (`apps/<app>/target/`) is embedded.
    #[test]
    fn no_build_output_is_embedded() {
        for embedded in EMBEDDED {
            for file in embedded.files {
                let parts: Vec<&str> = file.path.split('/').collect();
                let in_target = parts.first() == Some(&"target")
                    || (parts.first() == Some(&"apps") && parts.get(2) == Some(&"target"));
                assert!(
                    !in_target,
                    "`{}` embeds build output: `{}`",
                    embedded.dir, file.path
                );
            }
        }
    }

    /// A JS App installed in place (`node_modules/`) or a developer's own
    /// `.env` never reaches the embedding; the template's `.env.example`
    /// does.
    #[test]
    fn no_install_or_local_environment_is_embedded() {
        for embedded in EMBEDDED {
            for file in embedded.files {
                let mut parts = file.path.split('/');
                assert!(
                    !parts.any(|part| part == "node_modules" || part == ".env"),
                    "`{}` embeds `{}`",
                    embedded.dir,
                    file.path
                );
            }
        }
        let monorepo = find("monorepo").unwrap_or_else(|| panic!("`monorepo` is not embedded"));
        assert!(
            monorepo
                .files
                .iter()
                .any(|file| file.path == "apps/web/.env.example"),
            "the web App's `.env.example` is not embedded"
        );
    }
}
