//! Keeps the templates' guide (`docs/guide/`, its recipes too) honest: a
//! fenced block right after `<!-- source: <path> -->` must appear, verbatim,
//! in that file (a path from this crate's root, such as
//! `templates/axum-api/src/app/run.rs`), so a sample can't drift from the
//! code it quotes. A Rust block needs that marker, or `<!-- not compiled:
//! <reason> -->` (a recipe's code, which the reader adapts).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Reads a text file with line endings normalized to `\n`.
fn read_text(path: &Path) -> Result<String> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(text.replace("\r\n", "\n"))
}

/// Every Markdown page under `dir`, by its path from this crate's root.
fn pages(dir: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            pages(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let name = path
                .strip_prefix(root())?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((name, read_text(&path)?));
        }
    }
    Ok(())
}

/// One fenced block: its language, the `<!-- … -->` on the line before the
/// fence (without the markers), its body, and the fence's 1-based line.
struct Block {
    lang: String,
    marker: Option<String>,
    body: String,
    line: usize,
}

/// The fenced blocks of `text`, a list item's indented ones too (their
/// body without the fence's indent).
fn blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut previous: Option<&str> = None;
    let mut open: Option<(Block, usize)> = None;
    for (index, line) in text.lines().enumerate() {
        match open.take() {
            Some((block, _)) if line.trim() == "```" => blocks.push(block),
            Some((mut block, indent)) => {
                let unindented = line
                    .char_indices()
                    .find(|(at, c)| *at >= indent || *c != ' ')
                    .map_or("", |(at, _)| &line[at..]);
                block.body.push_str(unindented);
                block.body.push('\n');
                open = Some((block, indent));
            }
            None => {
                let indent = line.len() - line.trim_start().len();
                if let Some(lang) = line.trim_start().strip_prefix("```") {
                    let marker = previous
                        .map(str::trim)
                        .and_then(|prev| prev.strip_prefix("<!--"))
                        .and_then(|prev| prev.strip_suffix("-->"))
                        .map(|marker| marker.trim().to_owned());
                    open = Some((
                        Block {
                            lang: lang.trim().to_owned(),
                            marker,
                            body: String::new(),
                            line: index + 1,
                        },
                        indent,
                    ));
                }
            }
        }
        previous = Some(line);
    }
    blocks
}

#[test]
fn every_sample_comes_from_its_source() -> Result<()> {
    let mut documents = Vec::new();
    pages(&root().join("docs/guide"), &mut documents)?;
    ensure!(!documents.is_empty(), "docs/guide holds no page");
    let mut problems = Vec::new();
    let mut sourced = 0;
    for (name, document) in &documents {
        for block in blocks(document) {
            let marker = block.marker.as_deref().unwrap_or_default();
            if let Some(source) = marker.strip_prefix("source:").map(str::trim) {
                let text = read_text(&root().join(source))?;
                if text.contains(&block.body) {
                    sourced += 1;
                } else {
                    problems.push(format!(
                        "{name}:{}: the block is not in {source}:\n{}",
                        block.line, block.body
                    ));
                }
            } else if block.lang == "rust" && !marker.starts_with("not compiled:") {
                problems.push(format!(
                    "{name}:{}: a rust block without `<!-- source: <path> -->` or \
                     `<!-- not compiled: <reason> -->`",
                    block.line
                ));
            }
        }
    }
    ensure!(problems.is_empty(), "{}", problems.join("\n\n"));
    ensure!(sourced > 0, "no sample of the guide names its source");
    Ok(())
}
