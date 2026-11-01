//! The renames a Monorepo template's files get at scaffold time (workspace
//! ADR 0027): every App named after the project and its org. The template
//! ships its Apps under fixed names — `axum-api`, `web`, `desktop`,
//! `mobile`, Flutter's `com.example.flutter_starter` — and [`Renames`]
//! says what each becomes, per App, from [`Names`].
//!
//! Each rule renames a whole identifier: a match counts only when the
//! characters around it are not identifier characters (ASCII letters,
//! digits, `_`, `-`), so `my-desktop` and `flutter_starters` stay. Code
//! files are matched by their language's segment conventions
//! (`package:mobile/` in Dart; Rust paths are the engine's crate rename,
//! driven by the renamed manifest), manifests by the whole value
//! (`name = "desktop"`, `"name": "web"`). All of a file's rules apply in
//! one pass, so a rule's output is never matched again.

use std::borrow::Cow;

use crate::names::Names;

/// Which template paths a rule applies to.
#[derive(Debug, Clone, Copy)]
enum Scope {
    /// Every file of the template.
    All,
    /// The files under this directory.
    Under(&'static str),
    /// These files.
    Files(&'static [&'static str]),
}

impl Scope {
    fn covers(self, path: &str) -> bool {
        match self {
            Self::All => true,
            Self::Under(dir) => path
                .strip_prefix(dir)
                .is_some_and(|rest| rest.starts_with('/')),
            Self::Files(files) => files.contains(&path),
        }
    }
}

/// One rename: `from`, as a whole identifier, becomes `to` in the files
/// `scope` covers.
#[derive(Debug, Clone)]
struct Rule {
    scope: Scope,
    from: &'static str,
    to: String,
}

/// The mobile App's Kotlin sources, under the directory Flutter names
/// after the template's Android package.
const KOTLIN_DIR: &str = "apps/mobile/android/app/src/main/kotlin/com/example/flutter_starter/";

/// The renames of one template's files for one project. A single-app
/// template's files are its project's already (the engine renames its
/// crate from the project's manifest) and get none.
#[derive(Debug, Clone, Default)]
pub struct Renames {
    rules: Vec<Rule>,
    /// The Kotlin directory's new path, for the monorepo template.
    kotlin_dir: Option<String>,
}

impl Renames {
    /// No renames: every file as the template ships it.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// The renames of the template `template`'s files for `names`.
    #[must_use]
    pub fn new(template: &str, names: &Names) -> Self {
        if template != "monorepo" {
            return Self::none();
        }
        let org = names.org();
        let snake = names.snake();
        let rule = |scope, from, to: String| Rule { scope, from, to };
        let mut rules = vec![
            // api: the package, wherever it is named — its manifest, the
            // root `Cargo.lock`, `cargo run -p` in the docs, `rok.toml`'s
            // export, and the document's title (`CARGO_PKG_NAME`).
            rule(Scope::All, "axum-api", names.api_package()),
            // desktop: the package, in its manifest, the lock and the docs.
            rule(
                Scope::Files(&["apps/desktop/Cargo.toml.template", "Cargo.lock"]),
                "name = \"desktop\"",
                format!("name = \"{}\"", names.desktop_package()),
            ),
            rule(
                Scope::All,
                "-p desktop",
                format!("-p {}", names.desktop_package()),
            ),
            // web: the package.
            rule(
                Scope::Files(&["apps/web/package.json"]),
                "\"name\": \"web\"",
                format!("\"name\": \"{}-web\"", names.kebab()),
            ),
            // mobile: the Dart package and its imports.
            rule(
                Scope::Files(&["apps/mobile/pubspec.yaml"]),
                "name: mobile",
                format!("name: {snake}_mobile"),
            ),
            rule(
                Scope::Under("apps/mobile"),
                "package:mobile/",
                format!("package:{snake}_mobile/"),
            ),
            // mobile: Android's applicationId and namespace, Linux's
            // application id (`<org>.<project>`); the iOS and macOS bundle
            // ids in Flutter's camel case (`<org>.<project>`).
            rule(
                Scope::Under("apps/mobile"),
                "com.example.flutter_starter",
                format!("{org}.{snake}"),
            ),
            rule(
                Scope::Under("apps/mobile"),
                "com.example.flutterStarter",
                format!("{org}.{}", names.camel()),
            ),
            // mobile: the display name — Android's label, iOS's display
            // name, the app's title.
            rule(
                Scope::Under("apps/mobile"),
                "android:label=\"flutter_starter\"",
                format!("android:label=\"{}\"", names.display()),
            ),
            rule(
                Scope::Under("apps/mobile"),
                "Flutter Starter",
                names.display().to_owned(),
            ),
            // mobile: the binary and product name of each platform.
            rule(
                Scope::Under("apps/mobile"),
                "flutter_starter",
                snake.to_owned(),
            ),
            // mobile: the org left alone (copyright and company lines).
            rule(Scope::Under("apps/mobile"), "com.example", org.to_owned()),
            // mobile: the deep-link scheme.
            rule(
                Scope::Under("apps/mobile"),
                "myapp",
                names.scheme().to_owned(),
            ),
        ];
        // At one position the longest `from` wins: `com.example` must not
        // take `com.example.flutter_starter`'s place.
        rules.sort_by_key(|rule| std::cmp::Reverse(rule.from.len()));
        Self {
            rules,
            kotlin_dir: Some(format!(
                "apps/mobile/android/app/src/main/kotlin/{}/{snake}/",
                org.replace('.', "/")
            )),
        }
    }

    /// The text of the template file `path` (its path in the template,
    /// such as `apps/api/Cargo.toml.template`), renamed. The root
    /// `Cargo.lock` keeps its packages sorted by name, as cargo writes it,
    /// so `--locked` builds accept it.
    #[must_use]
    pub fn text<'a>(&self, path: &str, text: &'a str) -> Cow<'a, str> {
        let rules: Vec<&Rule> = self
            .rules
            .iter()
            .filter(|rule| rule.scope.covers(path))
            .collect();
        if rules.is_empty() {
            return Cow::Borrowed(text);
        }
        let renamed = replace_identifiers(text, &rules);
        if path == "Cargo.lock" && renamed != text {
            return Cow::Owned(sort_lock_packages(&renamed));
        }
        if renamed == text {
            Cow::Borrowed(text)
        } else {
            Cow::Owned(renamed)
        }
    }

    /// The project path `destination` (a template file's, after
    /// [`crate::catalog::destination`]), renamed: the mobile App's Kotlin
    /// sources move to the directory its Android package names.
    #[must_use]
    pub fn path<'a>(&self, destination: &'a str) -> Cow<'a, str> {
        match (&self.kotlin_dir, destination.strip_prefix(KOTLIN_DIR)) {
            (Some(dir), Some(rest)) => Cow::Owned(format!("{dir}{rest}")),
            _ => Cow::Borrowed(destination),
        }
    }
}

/// Whether `c` continues an identifier: a match next to one is part of a
/// longer name.
fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// `text` with every whole-identifier match of a rule replaced, in one
/// pass; at one position the first rule that matches wins.
fn replace_identifiers(text: &str, rules: &[&Rule]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    let mut previous: Option<char> = None;
    while at < text.len() {
        let rest = &text[at..];
        let after_identifier = previous.is_some_and(is_identifier_char);
        // A rule's edge that is not an identifier character (the `/` of
        // `package:mobile/`) is a boundary of its own.
        let matched = rules.iter().find(|rule| {
            let opens = rule.from.chars().next().is_some_and(is_identifier_char);
            let closes = rule
                .from
                .chars()
                .next_back()
                .is_some_and(is_identifier_char);
            rest.starts_with(rule.from)
                && !(opens && after_identifier)
                && !(closes
                    && rest[rule.from.len()..]
                        .chars()
                        .next()
                        .is_some_and(is_identifier_char))
        });
        if let Some(rule) = matched {
            out.push_str(&rule.to);
            at += rule.from.len();
            previous = rule.from.chars().next_back();
            continue;
        }
        let Some(c) = rest.chars().next() else {
            break;
        };
        out.push(c);
        at += c.len_utf8();
        previous = Some(c);
    }
    out
}

/// A `Cargo.lock` with its `[[package]]` entries sorted by name, as cargo
/// writes them; each entry keeps its place among those of its own name. A
/// lock not laid out as cargo lays it out is returned as it is.
fn sort_lock_packages(lock: &str) -> String {
    const PACKAGE: &str = "[[package]]";
    let Some(start) = lock.find(PACKAGE) else {
        return lock.to_owned();
    };
    let (head, body) = lock.split_at(start);
    let mut entries: Vec<&str> = body.trim_end_matches('\n').split("\n\n").collect();
    if !entries.iter().all(|entry| entry.starts_with(PACKAGE)) {
        return lock.to_owned();
    }
    let name = |entry: &str| -> String {
        entry
            .lines()
            .find_map(|line| line.strip_prefix("name = "))
            .unwrap_or_default()
            .trim_matches('"')
            .to_owned()
    };
    entries.sort_by_key(|entry| name(entry));
    format!("{head}{}\n", entries.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn renames() -> Renames {
        let names = Names::new("my shop", "com.acme").unwrap_or_else(|err| panic!("{err}"));
        Renames::new("monorepo", &names)
    }

    #[test]
    fn the_api_apps_package_is_renamed_wherever_it_is_named() {
        let renames = renames();
        assert_eq!(
            renames.text(
                "apps/api/Cargo.toml.template",
                "[package]\nname = \"axum-api\"\n"
            ),
            "[package]\nname = \"my-shop-api\"\n"
        );
        assert_eq!(
            renames.text(
                "rok.toml",
                "export = [\"cargo\", \"run\", \"-p\", \"axum-api\", \"--\"]\n"
            ),
            "export = [\"cargo\", \"run\", \"-p\", \"my-shop-api\", \"--\"]\n"
        );
        assert_eq!(
            renames.text("apps/api/openapi.json", "    \"title\": \"axum-api\",\n"),
            "    \"title\": \"my-shop-api\",\n"
        );
        assert_eq!(
            renames.text(
                "README.md",
                "cargo run -p axum-api\n# my-axum-api, axum-api2\n"
            ),
            "cargo run -p my-shop-api\n# my-axum-api, axum-api2\n"
        );
    }

    #[test]
    fn the_desktop_apps_package_is_renamed_in_its_manifest_lock_and_docs() {
        let renames = renames();
        assert_eq!(
            renames.text(
                "apps/desktop/Cargo.toml.template",
                "[package]\nname = \"desktop\"\n"
            ),
            "[package]\nname = \"my-shop-desktop\"\n"
        );
        assert_eq!(
            renames.text(
                "apps/desktop/README.md",
                "cargo test -p desktop\nthe desktop App, `name = \"desktop\"`\n"
            ),
            "cargo test -p my-shop-desktop\nthe desktop App, `name = \"desktop\"`\n"
        );
    }

    #[test]
    fn the_web_apps_package_is_renamed_in_its_manifest_only() {
        let renames = renames();
        assert_eq!(
            renames.text(
                "apps/web/package.json",
                "{\n  \"name\": \"web\",\n  \"private\": true\n}\n"
            ),
            "{\n  \"name\": \"my-shop-web\",\n  \"private\": true\n}\n"
        );
        assert_eq!(
            renames.text("apps/web/src/x.json", "{\"name\": \"web\"}"),
            "{\"name\": \"web\"}"
        );
    }

    #[test]
    fn the_mobile_apps_package_ids_display_name_and_scheme_are_renamed() {
        let renames = renames();
        assert_eq!(
            renames.text("apps/mobile/pubspec.yaml", "name: mobile\ndescription: x\n"),
            "name: my_shop_mobile\ndescription: x\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/test/a_test.dart",
                "import 'package:mobile/app.dart';\nimport 'package:mobile_x/y.dart';\n"
            ),
            "import 'package:my_shop_mobile/app.dart';\nimport 'package:mobile_x/y.dart';\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/android/app/build.gradle.kts",
                "namespace = \"com.example.flutter_starter\"\napplicationId = \"com.example.flutter_starter\"\n"
            ),
            "namespace = \"com.acme.my_shop\"\napplicationId = \"com.acme.my_shop\"\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/ios/Runner.xcodeproj/project.pbxproj",
                "PRODUCT_BUNDLE_IDENTIFIER = com.example.flutterStarter.RunnerTests;\n"
            ),
            "PRODUCT_BUNDLE_IDENTIFIER = com.acme.myShop.RunnerTests;\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/android/app/src/main/AndroidManifest.xml",
                "<application android:label=\"flutter_starter\">\n<data android:scheme=\"myapp\" android:host=\"auth\"/>\n"
            ),
            "<application android:label=\"My Shop\">\n<data android:scheme=\"myshop\" android:host=\"auth\"/>\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/ios/Runner/Info.plist",
                "<string>Flutter Starter</string>\n<string>flutter_starter</string>\n"
            ),
            "<string>My Shop</string>\n<string>my_shop</string>\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/macos/Runner/Configs/AppInfo.xcconfig",
                "PRODUCT_COPYRIGHT = Copyright 2026 com.example. All rights reserved.\n"
            ),
            "PRODUCT_COPYRIGHT = Copyright 2026 com.acme. All rights reserved.\n"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/lib/app/core/config/app_config.dart",
                "defaultValue: 'myapp', // myapp://auth, not myapps\n"
            ),
            "defaultValue: 'myshop', // myshop://auth, not myapps\n"
        );
        // Outside the mobile App, its names are other things.
        assert_eq!(
            renames.text("apps/web/README.md", "myapp, com.example, mobile"),
            "myapp, com.example, mobile"
        );
    }

    #[test]
    fn the_mobile_apps_kotlin_sources_move_to_its_package_directory() {
        let renames = renames();
        assert_eq!(
            renames.path(
                "apps/mobile/android/app/src/main/kotlin/com/example/flutter_starter/MainActivity.kt"
            ),
            "apps/mobile/android/app/src/main/kotlin/com/acme/my_shop/MainActivity.kt"
        );
        assert_eq!(
            renames.path("apps/mobile/lib/main.dart"),
            "apps/mobile/lib/main.dart"
        );
        assert_eq!(
            renames.text(
                "apps/mobile/android/app/src/main/kotlin/com/example/flutter_starter/MainActivity.kt",
                "package com.example.flutter_starter\n"
            ),
            "package com.acme.my_shop\n"
        );
    }

    #[test]
    fn the_root_lock_stays_sorted_by_package_name() {
        let lock = "# header\nversion = 4\n\n\
                    [[package]]\nname = \"axum\"\nversion = \"0.8.0\"\n\n\
                    [[package]]\nname = \"axum-api\"\nversion = \"0.1.0\"\n\n\
                    [[package]]\nname = \"desktop\"\nversion = \"0.1.0\"\n\n\
                    [[package]]\nname = \"zip\"\nversion = \"1.0.0\"\n";
        assert_eq!(
            renames().text("Cargo.lock", lock),
            "# header\nversion = 4\n\n\
             [[package]]\nname = \"axum\"\nversion = \"0.8.0\"\n\n\
             [[package]]\nname = \"my-shop-api\"\nversion = \"0.1.0\"\n\n\
             [[package]]\nname = \"my-shop-desktop\"\nversion = \"0.1.0\"\n\n\
             [[package]]\nname = \"zip\"\nversion = \"1.0.0\"\n"
        );
        let names = Names::new("aaa", "com.acme").unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            Renames::new("monorepo", &names).text("Cargo.lock", lock),
            "# header\nversion = 4\n\n\
             [[package]]\nname = \"aaa-api\"\nversion = \"0.1.0\"\n\n\
             [[package]]\nname = \"aaa-desktop\"\nversion = \"0.1.0\"\n\n\
             [[package]]\nname = \"axum\"\nversion = \"0.8.0\"\n\n\
             [[package]]\nname = \"zip\"\nversion = \"1.0.0\"\n"
        );
    }

    /// Renamed, the monorepo template names none of the names it ships
    /// with: every App is the project's.
    #[test]
    fn the_renamed_monorepo_keeps_none_of_the_templates_names() {
        let renames = renames();
        let files = crate::catalog::files("monorepo").unwrap_or_else(|| panic!("no monorepo"));
        let mut left = Vec::new();
        for file in files {
            let Ok(text) = std::str::from_utf8(file.bytes) else {
                continue;
            };
            let text = renames.text(file.path, text);
            let mut shipped = vec![
                "axum-api",
                "flutter_starter",
                "flutterStarter",
                "Flutter Starter",
                "package:mobile/",
                "name = \"desktop\"",
                "-p desktop",
            ];
            if file.path.starts_with("apps/mobile/") {
                shipped.extend(["com.example", "myapp"]);
            }
            // A Rust file's `axum_api::` paths are the engine's to rename,
            // from the renamed manifest; anywhere else the name is left over.
            if !file.path.ends_with(".rs") {
                shipped.push("axum_api");
            }
            for name in shipped {
                if text.contains(name) {
                    left.push(format!("{}: {name}", file.path));
                }
            }
        }
        assert!(left.is_empty(), "left after renaming:\n{}", left.join("\n"));
    }

    #[test]
    fn a_single_app_template_gets_no_renames() {
        let names = Names::new("shop", "com.acme").unwrap_or_else(|err| panic!("{err}"));
        let renames = Renames::new("axum", &names);
        assert_eq!(
            renames.text("Cargo.toml.template", "name = \"axum-api\"\n"),
            "name = \"axum-api\"\n"
        );
    }

    /// The default org renames nothing of the org's own: `com.example`
    /// stays, the project's name replaces Flutter's.
    #[test]
    fn the_default_org_keeps_com_example() {
        let names =
            Names::new("shop", crate::names::DEFAULT_ORG).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            Renames::new("monorepo", &names).text(
                "apps/mobile/linux/CMakeLists.txt",
                "set(APPLICATION_ID \"com.example.flutter_starter\")\n"
            ),
            "set(APPLICATION_ID \"com.example.shop\")\n"
        );
    }
}
