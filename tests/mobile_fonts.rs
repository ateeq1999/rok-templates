//! The monorepo's mobile App bundles its fonts (so its golden test draws
//! the same glyphs on every machine), and only fonts under the SIL Open Font
//! License, each shipped with its licence: an `OFL.txt` beside it that is
//! the OFL, and that `pubspec.yaml` bundles into the App.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The mobile App of the monorepo template.
fn mobile() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/monorepo/apps/mobile")
}

/// The font files under `dir`, at any depth.
fn fonts(dir: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        if path.is_dir() {
            fonts(&path, found)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf"))
        {
            found.push(path);
        }
    }
    Ok(())
}

#[test]
fn every_bundled_font_ships_with_its_ofl_licence() -> Result<()> {
    let mobile = mobile();
    let pubspec = fs::read_to_string(mobile.join("pubspec.yaml")).context("pubspec.yaml")?;
    let mut found = Vec::new();
    fonts(&mobile.join("fonts"), &mut found)?;
    assert!(!found.is_empty(), "apps/mobile/fonts/ bundles no font");
    for font in &found {
        let relative = font
            .strip_prefix(&mobile)?
            .to_string_lossy()
            .replace('\\', "/");
        assert!(
            pubspec.contains(&format!("- asset: {relative}")),
            "apps/mobile/pubspec.yaml does not declare {relative}"
        );
        let dir = font.parent().context("a font's directory")?;
        let licence_path = dir.join("OFL.txt");
        let licence = fs::read_to_string(&licence_path)
            .with_context(|| format!("{relative} has no OFL.txt beside it"))?;
        assert!(
            licence.contains("SIL OPEN FONT LICENSE Version 1.1"),
            "{} is not the SIL Open Font License 1.1",
            licence_path.display()
        );
        let licence_relative = licence_path
            .strip_prefix(&mobile)?
            .to_string_lossy()
            .replace('\\', "/");
        assert!(
            pubspec.contains(&format!("- {licence_relative}")),
            "apps/mobile/pubspec.yaml does not bundle {licence_relative} into the App"
        );
    }
    Ok(())
}
