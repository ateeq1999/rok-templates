//! The names a Monorepo template's Apps take from the project (workspace
//! ADR 0027): `init` derives them from the project's name and `--org`, and
//! [`crate::rename`] writes them into the Apps.

use std::fmt;

/// The org `--org` defaults to.
pub const DEFAULT_ORG: &str = "com.example";

/// The names a project's Apps are given, each in the form its ecosystem
/// takes, from the project's name (its directory's) and its org.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Names {
    kebab: String,
    snake: String,
    camel: String,
    display: String,
    scheme: String,
    org: String,
}

/// Why a project name or an org cannot name the Apps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameError {
    /// The project's name holds no letter or digit.
    EmptyProject(String),
    /// The project's name starts with a digit.
    ProjectStartsWithDigit(String),
    /// The project's name is a word Java, Kotlin or Dart reserves, so it
    /// cannot be a segment of the Android package.
    ReservedProject(String),
    /// The org is not a reverse-domain name of two or more segments, each
    /// a lowercase ASCII letter followed by lowercase letters and digits.
    InvalidOrg(String),
    /// A segment of the org is a word Java or Kotlin reserves.
    ReservedOrg(String),
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProject(name) => write!(
                f,
                "the project name `{name}` has no letter or digit to name the Apps after"
            ),
            Self::ProjectStartsWithDigit(name) => write!(
                f,
                "the project name `{name}` starts with a digit; the Apps' package names must start with a letter"
            ),
            Self::ReservedProject(name) => write!(
                f,
                "the project name `{name}` is a reserved word; it cannot name the mobile App's package"
            ),
            Self::InvalidOrg(org) => write!(
                f,
                "`--org {org}` is not a reverse-domain name like `com.example`: two or more \
                 dot-separated segments, each a lowercase letter followed by lowercase letters or digits"
            ),
            Self::ReservedOrg(org) => write!(
                f,
                "`--org {org}` has a reserved word as a segment; it cannot name the mobile App's package"
            ),
        }
    }
}

impl std::error::Error for NameError {}

/// Words Java, Kotlin or Dart reserve: none can be a segment of an Android
/// package (`<org>.<project>`) or a Kotlin `package` line.
const RESERVED: &[&str] = &[
    "abstract",
    "as",
    "assert",
    "boolean",
    "break",
    "byte",
    "case",
    "catch",
    "char",
    "class",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "for",
    "fun",
    "goto",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "int",
    "interface",
    "is",
    "long",
    "native",
    "new",
    "null",
    "object",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "short",
    "static",
    "strictfp",
    "super",
    "switch",
    "synchronized",
    "this",
    "throw",
    "throws",
    "transient",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "void",
    "volatile",
    "when",
    "while",
];

impl Names {
    /// The names for the project `project` (its directory's name, as
    /// typed) under the org `org`. The project's name is split into words
    /// at every character that is not an ASCII letter or digit, and each
    /// word lowercased: `My Shop` and `my_shop` are both `my-shop`.
    ///
    /// # Errors
    ///
    /// When either would break an identifier: see [`NameError`].
    pub fn new(project: &str, org: &str) -> Result<Self, NameError> {
        let words: Vec<String> = project
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(str::to_ascii_lowercase)
            .collect();
        let Some(first) = words.first() else {
            return Err(NameError::EmptyProject(project.to_owned()));
        };
        if first.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(NameError::ProjectStartsWithDigit(project.to_owned()));
        }
        let snake = words.join("_");
        if RESERVED.contains(&snake.as_str()) {
            return Err(NameError::ReservedProject(project.to_owned()));
        }
        let segments: Vec<&str> = org.split('.').collect();
        let valid_segment = |segment: &&str| {
            segment.starts_with(|c: char| c.is_ascii_lowercase())
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        };
        if segments.len() < 2 || !segments.iter().all(valid_segment) {
            return Err(NameError::InvalidOrg(org.to_owned()));
        }
        if segments.iter().any(|segment| RESERVED.contains(segment)) {
            return Err(NameError::ReservedOrg(org.to_owned()));
        }
        let capitalized = |word: &String| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |c| {
                c.to_ascii_uppercase().to_string() + chars.as_str()
            })
        };
        Ok(Self {
            kebab: words.join("-"),
            camel: words
                .iter()
                .enumerate()
                .map(|(at, word)| {
                    if at == 0 {
                        word.clone()
                    } else {
                        capitalized(word)
                    }
                })
                .collect(),
            display: words.iter().map(capitalized).collect::<Vec<_>>().join(" "),
            scheme: words.concat(),
            snake,
            org: org.to_owned(),
        })
    }

    /// The project's name in kebab case, as Cargo and npm packages take
    /// it: `my-shop`.
    #[must_use]
    pub fn kebab(&self) -> &str {
        &self.kebab
    }

    /// The project's name in snake case, as Rust crates, Dart packages and
    /// Android package segments take it: `my_shop`.
    #[must_use]
    pub fn snake(&self) -> &str {
        &self.snake
    }

    /// The project's name in lower camel case, as an iOS bundle id's last
    /// segment takes it (Flutter's own convention): `myShop`.
    #[must_use]
    pub fn camel(&self) -> &str {
        &self.camel
    }

    /// The name a user sees, its words capitalized: `My Shop`.
    #[must_use]
    pub fn display(&self) -> &str {
        &self.display
    }

    /// The deep-link scheme: the project's letters and digits, lowercase,
    /// a valid URL scheme on every platform: `myshop`.
    #[must_use]
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// The org, a reverse-domain name: `com.example`.
    #[must_use]
    pub fn org(&self) -> &str {
        &self.org
    }

    /// The api App's package: `my-shop-api`.
    #[must_use]
    pub fn api_package(&self) -> String {
        format!("{}-api", self.kebab)
    }

    /// The desktop App's package: `my-shop-desktop`.
    #[must_use]
    pub fn desktop_package(&self) -> String {
        format!("{}-desktop", self.kebab)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(project: &str, org: &str) -> Names {
        Names::new(project, org).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn a_one_word_project_names_every_form_alike() {
        let names = names("shop", "com.acme");
        assert_eq!(names.kebab(), "shop");
        assert_eq!(names.snake(), "shop");
        assert_eq!(names.camel(), "shop");
        assert_eq!(names.display(), "Shop");
        assert_eq!(names.scheme(), "shop");
        assert_eq!(names.org(), "com.acme");
        assert_eq!(names.api_package(), "shop-api");
        assert_eq!(names.desktop_package(), "shop-desktop");
    }

    #[test]
    fn a_project_splits_into_words_at_every_separator() {
        for project in ["my-shop", "my_shop", "My Shop", ".My.Shop!"] {
            let names = names(project, DEFAULT_ORG);
            assert_eq!(names.kebab(), "my-shop", "{project}");
            assert_eq!(names.snake(), "my_shop", "{project}");
            assert_eq!(names.camel(), "myShop", "{project}");
            assert_eq!(names.display(), "My Shop", "{project}");
            assert_eq!(names.scheme(), "myshop", "{project}");
        }
    }

    #[test]
    fn a_project_that_breaks_an_identifier_is_rejected() {
        assert_eq!(
            Names::new("--", DEFAULT_ORG),
            Err(NameError::EmptyProject("--".to_owned()))
        );
        assert_eq!(
            Names::new("2shop", DEFAULT_ORG),
            Err(NameError::ProjectStartsWithDigit("2shop".to_owned()))
        );
        assert_eq!(
            Names::new("new", DEFAULT_ORG),
            Err(NameError::ReservedProject("new".to_owned()))
        );
    }

    #[test]
    fn the_org_must_be_a_lowercase_reverse_domain_name() {
        for org in ["com.example", "io.acme2", "uk.co.shop"] {
            assert!(Names::new("shop", org).is_ok(), "{org}");
        }
        for org in [
            "acme",
            "com.",
            ".com.acme",
            "com..acme",
            "com.Acme",
            "com.my-co",
            "com.my_co",
            "com.2acme",
            "",
        ] {
            assert_eq!(
                Names::new("shop", org),
                Err(NameError::InvalidOrg(org.to_owned())),
                "{org}"
            );
        }
        assert_eq!(
            Names::new("shop", "com.new"),
            Err(NameError::ReservedOrg("com.new".to_owned()))
        );
    }
}
