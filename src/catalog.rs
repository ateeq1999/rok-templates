//! The catalog: which template names exist, and what a project gets for
//! one (workspace ADR 0027, amending 0022's data-only crate). `init
//! --template <NAME>` takes a [`names()`] value; [`files`] resolves it to
//! the template's embedded files, and [`destination`] maps a file's path
//! in the template to its path in the project. The grammar (`rok sqlc
//! init`) and the write policy — copy when absent, skip files the engine's
//! generator marked `@generated` — stay the engine's
//! (`rok-data-sqlc`'s `init`), so the marker rules stay with the engine.

use crate::File;

/// One App of a Monorepo template: its name, and where its files sit
/// under the template's directory.
#[derive(Debug, Clone, Copy)]
pub struct App {
    /// The App's name, as the scaffold's `rok.toml` names it in `[apps]`.
    pub name: &'static str,
    /// The directory of `templates/`, under the template's, the App's
    /// files sit in.
    pub dir: &'static str,
}

/// A template `--template` names: its name in the grammar, and the
/// `templates/` directory resolved for it.
#[derive(Debug, Clone, Copy)]
pub struct Template {
    /// The name `init --template` takes, such as `axum`.
    pub name: &'static str,
    /// The template directory this name resolves to.
    pub dir: &'static str,
    /// The template's Apps. A Monorepo template lists them; a single-app
    /// template is its one App's files and lists none.
    pub apps: &'static [App],
}

/// The Apps of the monorepo template. The web and mobile Apps join as
/// their files arrive.
const MONOREPO_APPS: &[App] = &[App {
    name: "api",
    dir: "apps/api",
}];

/// The templates, each its [`Template`] entry. A template's manifest and
/// every file a project gets is one of [`files`]'s, laid into the project
/// at [`destination`] paths.
pub const TEMPLATES: &[Template] = &[
    Template {
        name: "actix",
        dir: "actix-api",
        apps: &[],
    },
    Template {
        name: "axum",
        dir: "axum-api",
        apps: &[],
    },
    Template {
        name: "monorepo",
        dir: "monorepo",
        apps: MONOREPO_APPS,
    },
];

/// The names `--template` takes, in catalog order.
pub fn names() -> impl Iterator<Item = &'static str> {
    TEMPLATES.iter().map(|template| template.name)
}

/// The template `name`, or `None` when the catalog names no such template
/// (the engine's `init` turns that into its usage error).
#[must_use]
pub fn template(name: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|template| template.name == name)
}

/// The embedded files of the template `name` — every file but a build
/// directory's, sorted by path — or `None` when the catalog names no such
/// template.
#[must_use]
pub fn files(name: &str) -> Option<&'static [File]> {
    template(name)
        .and_then(|template| crate::find(template.dir))
        .map(|embedded| embedded.files)
}

/// The template's Apps, empty for a single-app template.
#[must_use]
pub fn apps(name: &str) -> &'static [App] {
    template(name).map_or(&[], |template| template.apps)
}

/// The suffix [`destination`] strips.
const TEMPLATE_SUFFIX: &str = ".template";

/// Where a template file goes in a project: its own path, but
/// `<path>.template` becomes `<path>`. Cargo leaves a directory holding a
/// `Cargo.toml` (another package) out of a published crate, so a template's
/// manifest is checked in as `Cargo.toml.template`.
#[must_use]
pub fn destination(path: &str) -> &str {
    path.strip_suffix(TEMPLATE_SUFFIX).unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EMBEDDED;

    #[test]
    fn every_name_resolves_to_a_directed_embedded_template() {
        for name in names() {
            let template = template(name).unwrap_or_else(|| panic!("`{name}` not in the catalog"));
            assert!(
                EMBEDDED.iter().any(|embedded| embedded.dir == template.dir),
                "catalog name `{name}` resolves to `{}`, which is not embedded",
                template.dir
            );
            let files =
                files(name).unwrap_or_else(|| panic!("template `{name}` resolves to no files"));
            assert!(!files.is_empty(), "template `{name}` has no files");
            assert!(
                files
                    .iter()
                    .any(|file| destination(file.path) == "Cargo.toml"),
                "template `{name}` has no manifest among its files"
            );
        }
    }

    #[test]
    fn every_template_directory_is_in_the_catalog() {
        for embedded in EMBEDDED {
            let known = TEMPLATES
                .iter()
                .any(|template| template.dir == embedded.dir);
            assert!(
                known,
                "`{}` is embedded but not in the catalog",
                embedded.dir
            );
        }
    }

    #[test]
    fn names_are_unique() {
        let names: Vec<&str> = names().collect();
        let count = names.len();
        assert_eq!(
            count,
            names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            "the catalog names a template twice"
        );
    }

    #[test]
    fn destination_strips_the_manifest_suffix_only() {
        assert_eq!(destination("Cargo.toml.template"), "Cargo.toml");
        assert_eq!(destination("src/lib.rs"), "src/lib.rs");
        assert_eq!(
            destination("deploy/a.template/x.template"),
            "deploy/a.template/x"
        );
    }

    #[test]
    fn a_monorepo_template_names_embedded_apps() {
        let named = apps("monorepo");
        assert!(
            !named.is_empty(),
            "the monorepo template names no Apps (it is a Monorepo template)"
        );
        for app in named {
            let files = files("monorepo")
                .unwrap_or_else(|| panic!("the monorepo template has no files"))
                .iter()
                .filter(|file| file.path.starts_with(app.dir))
                .map(|file| {
                    file.path
                        .strip_prefix(&format!("{}/", app.dir))
                        .expect("the prefix matched")
                })
                .collect::<Vec<_>>();
            assert!(
                !files.is_empty(),
                "App `{}` (`{}`) has no files in the template",
                app.name,
                app.dir
            );
            assert!(
                files.contains(&"Cargo.toml.template"),
                "App `{}` has no manifest: {:?}",
                app.name,
                files.iter().find(|path| path.ends_with("Cargo"))
            );
        }
        assert!(
            apps("actix").is_empty(),
            "a single-app template is not a monorepo"
        );
    }

    #[test]
    fn the_catalog_names_its_templates() {
        let names: Vec<&str> = names().collect();
        assert!(names.contains(&"actix"), "the catalog lost `actix`");
        assert!(names.contains(&"axum"), "the catalog lost `axum`");
    }
}
