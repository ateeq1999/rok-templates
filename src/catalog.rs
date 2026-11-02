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
    /// The App's name. An App `rok gen` regenerates (one with generation
    /// settings of its own, such as `api`) goes by it in the scaffold's
    /// `rok.toml` `[apps]`; the web, desktop and mobile Apps have nothing
    /// generated yet and are not listed there.
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

/// The Apps of the monorepo template.
const MONOREPO_APPS: &[App] = &[
    App {
        name: "api",
        dir: "apps/api",
    },
    App {
        name: "web",
        dir: "apps/web",
    },
    App {
        name: "desktop",
        dir: "apps/desktop",
    },
    App {
        name: "mobile",
        dir: "apps/mobile",
    },
];

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
                files.contains(&"Cargo.toml.template")
                    || files.contains(&"package.json")
                    || files.contains(&"pubspec.yaml"),
                "App `{}` has no manifest (`Cargo.toml.template`, `package.json` or `pubspec.yaml`)",
                app.name
            );
        }
        assert!(
            apps("actix").is_empty(),
            "a single-app template is not a monorepo"
        );
    }

    /// A JS App's package is named after the App the catalog names: its
    /// `package.json` declares `"name": "<app>"`, which init renames
    /// after the project ([`crate::rename`], workspace ADR 0027).
    #[test]
    fn a_js_apps_package_is_named_after_its_app() {
        let files =
            files("monorepo").unwrap_or_else(|| panic!("the monorepo template has no files"));
        for app in apps("monorepo") {
            let manifest = format!("{}/package.json", app.dir);
            let Some(file) = files.iter().find(|file| file.path == manifest) else {
                continue;
            };
            let text = String::from_utf8_lossy(file.bytes);
            assert!(
                text.contains(&format!("\"name\": \"{}\",", app.name)),
                "`{manifest}` does not name its package `{}`",
                app.name
            );
        }
    }

    /// A Flutter App's package is named after the App the catalog names:
    /// its `pubspec.yaml` declares `name: <app>`, which init renames
    /// after the project ([`crate::rename`], workspace ADR 0027).
    #[test]
    fn a_flutter_apps_package_is_named_after_its_app() {
        let files =
            files("monorepo").unwrap_or_else(|| panic!("the monorepo template has no files"));
        for app in apps("monorepo") {
            let manifest = format!("{}/pubspec.yaml", app.dir);
            let Some(file) = files.iter().find(|file| file.path == manifest) else {
                continue;
            };
            let text = String::from_utf8_lossy(file.bytes);
            assert!(
                text.lines()
                    .any(|line| line.trim_end() == format!("name: {}", app.name)),
                "`{manifest}` does not name its package `{}`",
                app.name
            );
        }
    }

    /// The monorepo's Apps: the API, and the web, desktop and mobile Apps
    /// that call it.
    #[test]
    fn the_monorepo_names_its_four_apps() {
        let named: Vec<(&str, &str)> = apps("monorepo")
            .iter()
            .map(|app| (app.name, app.dir))
            .collect();
        assert_eq!(
            named,
            [
                ("api", "apps/api"),
                ("web", "apps/web"),
                ("desktop", "apps/desktop"),
                ("mobile", "apps/mobile"),
            ]
        );
    }

    /// Every Rust App of the monorepo (one with a `Cargo.toml.template`) is
    /// a member of the root workspace, so `cargo -p <package>` reaches it
    /// from the root and the root `Cargo.lock` covers it.
    #[test]
    fn every_rust_app_is_a_member_of_the_root_workspace() {
        let files =
            files("monorepo").unwrap_or_else(|| panic!("the monorepo template has no files"));
        let root = files
            .iter()
            .find(|file| file.path == "Cargo.toml.template")
            .map(|file| String::from_utf8_lossy(file.bytes))
            .unwrap_or_else(|| panic!("the monorepo template has no root manifest"));
        let members = root
            .lines()
            .find(|line| line.starts_with("members"))
            .unwrap_or_else(|| panic!("the root manifest names no members:\n{root}"));
        for app in apps("monorepo") {
            let manifest = format!("{}/Cargo.toml.template", app.dir);
            if files.iter().any(|file| file.path == manifest) {
                assert!(
                    members.contains(&format!("\"{}\"", app.dir)),
                    "the Rust App `{}` is not a root workspace member: {members}",
                    app.name
                );
            }
        }
    }

    /// The desktop App ships as the package `desktop`, which init renames
    /// after the project ([`crate::rename`]).
    #[test]
    fn the_desktop_apps_package_is_named_desktop() {
        let manifest = files("monorepo")
            .unwrap_or_else(|| panic!("the monorepo template has no files"))
            .iter()
            .find(|file| file.path == "apps/desktop/Cargo.toml.template")
            .map(|file| String::from_utf8_lossy(file.bytes))
            .unwrap_or_else(|| panic!("the desktop App has no manifest"));
        assert!(
            manifest.contains("\nname = \"desktop\"\n"),
            "apps/desktop's package is not named `desktop`:\n{manifest}"
        );
    }

    /// The desktop App reaches the API only through the generated
    /// `clients/rust` (workspace ADR 0031): its manifest depends on the
    /// client by path, so a contract change `rok gen` regenerates breaks or
    /// flows through its build, and no source of its calls HTTP by hand.
    #[test]
    fn the_desktop_app_calls_the_api_only_through_clients_rust() {
        let files =
            files("monorepo").unwrap_or_else(|| panic!("the monorepo template has no files"));
        let manifest = files
            .iter()
            .find(|file| file.path == "apps/desktop/Cargo.toml.template")
            .map(|file| String::from_utf8_lossy(file.bytes))
            .unwrap_or_else(|| panic!("the desktop App has no manifest"));
        assert!(
            manifest.contains("\napi-client = { path = \"../../clients/rust\" }\n"),
            "apps/desktop does not depend on clients/rust by path:\n{manifest}"
        );
        for file in files
            .iter()
            .filter(|file| file.path.starts_with("apps/desktop/src/"))
        {
            let text = String::from_utf8_lossy(file.bytes);
            for by_hand in ["HttpClient", "http::Options", "reqwest"] {
                assert!(
                    !text.contains(by_hand),
                    "`{}` calls HTTP by hand (`{by_hand}`)",
                    file.path
                );
            }
        }
    }

    /// The api App's generated code (`src/db/mod.rs` among it, marked
    /// `@generated`) is embedded but never copied: init leaves it to `rok
    /// gen`, and `apps/api` does not build until that runs. So the
    /// README's first commands must run it.
    #[test]
    fn the_monorepo_readme_starts_with_rok_gen() {
        let files =
            files("monorepo").unwrap_or_else(|| panic!("the monorepo template has no files"));
        let db = files
            .iter()
            .find(|file| file.path == "apps/api/src/db/mod.rs")
            .map(|file| String::from_utf8_lossy(file.bytes))
            .unwrap_or_else(|| panic!("apps/api has no src/db/mod.rs"));
        assert!(
            db.lines()
                .next()
                .is_some_and(|line| line.contains("@generated by rok-data-sqlc")),
            "apps/api's `src/db/mod.rs` is no longer generated: this test's premise changed"
        );
        let readme = files
            .iter()
            .find(|file| file.path == "README.md")
            .map(|file| String::from_utf8_lossy(file.bytes).replace("\r\n", "\n"))
            .unwrap_or_else(|| panic!("the monorepo template has no README.md"));
        let start = readme
            .split("```sh\n")
            .nth(1)
            .and_then(|rest| rest.split("```").next())
            .unwrap_or_else(|| panic!("the monorepo README has no sh block"));
        let commands: Vec<&str> = start.lines().collect();
        assert_eq!(
            commands.first().copied(),
            Some("rok init --template monorepo my-app"),
            "{start}"
        );
        let build = readme.find("cargo run").unwrap_or(readme.len());
        let generate = readme.find("\nrok gen\n").unwrap_or(usize::MAX);
        assert!(
            commands.contains(&"rok gen") && generate < build,
            "the README's first block does not run `rok gen` before any build:\n{start}"
        );
    }

    #[test]
    fn the_catalog_names_its_templates() {
        let names: Vec<&str> = names().collect();
        assert!(names.contains(&"actix"), "the catalog lost `actix`");
        assert!(names.contains(&"axum"), "the catalog lost `axum`");
    }
}
