//! Embeds the project templates of `rok sqlc init --template` (the
//! directories of `templates/`) in the crate, so `init` works from
//! `cargo install`. It writes `$OUT_DIR/templates.rs`: one `Embedded` (see
//! `src/lib.rs`) per template, its directory and its files, each
//! a `File`: its path (relative, with `/`) and its bytes through
//! `include_bytes!`. What building or installing a template in place
//! leaves behind is left out: its top-level `target/`, an App's
//! (`apps/<app>/target/`, a Monorepo template's Rust App built in place)
//! and, at any depth, the directories of [`LOCAL_DIRS`]; so are the files
//! of [`LOCAL_FILES`] (OS junk, a developer's `.env`). Everything else, hidden files such as
//! `.env.example` included, is embedded, and `init` decides what it
//! copies. `Cargo.toml`'s `include` leaves out the same paths, so the
//! package ships what is embedded (`tests/package.rs` checks it).

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let root =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets it")).join("templates");
    println!("cargo:rerun-if-changed=templates");
    let mut output = String::from("&[\n");
    for template in sorted_entries(&root)? {
        if !template.is_dir() {
            continue;
        }
        let name = file_name(&template);
        output.push_str(&format!("    Embedded {{ dir: {name:?}, files: &[\n"));
        let mut files = Vec::new();
        collect(&template, &template, &mut files)?;
        files.sort();
        for (relative, absolute) in files {
            output.push_str(&format!(
                "        File {{ path: {relative:?}, bytes: include_bytes!({absolute:?}) }},\n"
            ));
        }
        output.push_str("    ] },\n");
    }
    output.push_str("]\n");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets it"));
    fs::write(out.join("templates.rs"), output)
}

/// `dir`'s entries, sorted; none when `dir` doesn't exist.
fn sorted_entries(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Directories a JS App's install or build writes in place, at any depth:
/// its dependencies and its build output (what the web App's `.gitignore`
/// names).
const LOCAL_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    "dist-ssr",
    ".output",
    ".tanstack",
    ".vinxi",
    ".wrangler",
];

/// Files that are a machine's, never a template's: OS junk, and a
/// developer's own environment (`.env`; the template ships `.env.example`).
const LOCAL_FILES: &[&str] = &[".DS_Store", "Thumbs.db", "desktop.ini", ".env"];

/// Every file under `dir` as (its path relative to `template`, with `/`,
/// and its absolute path), but those of `template`'s `target/` and its
/// Apps' (`apps/<app>/target/`), of the [`LOCAL_DIRS`] and the
/// [`LOCAL_FILES`].
fn collect(template: &Path, dir: &Path, files: &mut Vec<(String, String)>) -> io::Result<()> {
    let builds_here = dir == template || dir.parent() == Some(template.join("apps").as_path());
    for path in sorted_entries(dir)? {
        let name = file_name(&path);
        if path.is_dir() {
            if (builds_here && name == "target") || LOCAL_DIRS.contains(&name.as_str()) {
                continue;
            }
            collect(template, &path, files)?;
        } else if !LOCAL_FILES.contains(&name.as_str()) {
            let relative = path
                .strip_prefix(template)
                .map_err(io::Error::other)?
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            files.push((relative, path.to_string_lossy().into_owned()));
        }
    }
    Ok(())
}
