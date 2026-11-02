//! The `apps/desktop` App of the monorepo template (workspace ADR 0031),
//! made the way a user makes it: `rok init shop --template monorepo --org
//! com.acme` into a scratch directory, then, from its root, `cargo build -p
//! shop-desktop` and `cargo test -p shop-desktop` (the App, renamed after
//! the project; its bloc tests run on GPUI's test support, no window).
//! Skips with a printed note on Linux without the system libraries GPUI
//! builds against.

use std::process::Command;

use anyhow::{Result, ensure};

use crate::common::{self, cargo};

/// The desktop App's package in the gates' scaffold, renamed after the
/// project.
const DESKTOP_PACKAGE: &str = "shop-desktop";

/// The Debian packages that give GPUI's Linux build what it links against
/// (what the `desktop` CI job installs).
const LINUX_PACKAGES: &str = "libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
     libvulkan-dev libfontconfig-dev libx11-xcb-dev libxcb1-dev";

/// The pkg-config names of the libraries GPUI's Linux build needs.
const LINUX_LIBRARIES: &[&str] = &["xkbcommon", "xkbcommon-x11", "wayland-client", "fontconfig"];

/// Why GPUI cannot build here, or `None` when it can: on Linux, a library
/// `pkg-config` cannot find (or no `pkg-config` at all). Windows and macOS
/// need nothing beyond the toolchain.
fn gpui_skip_reason() -> Option<String> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let missing: Vec<&str> = LINUX_LIBRARIES
        .iter()
        .copied()
        .filter(|library| {
            !Command::new("pkg-config")
                .args(["--exists", library])
                .status()
                .is_ok_and(|status| status.success())
        })
        .collect();
    (!missing.is_empty()).then(|| {
        format!(
            "GPUI's Linux libraries are missing ({}, per pkg-config); \
             `sudo apt-get install {LINUX_PACKAGES}` installs them",
            missing.join(", ")
        )
    })
}

/// The gate: scaffold the monorepo, then build the desktop App and run its
/// tests from the workspace root.
#[test]
fn monorepo_desktop_app_builds_and_tests() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    if let Some(reason) = gpui_skip_reason() {
        eprintln!("note: skipping the desktop App gate: {reason}");
        return Ok(());
    }
    let project = crate::monorepo_template::scaffold()?;
    let root = project.path();
    ensure!(
        root.join("apps/desktop/Cargo.toml").is_file(),
        "the scaffold has no apps/desktop/Cargo.toml"
    );
    // The root `Cargo.lock` the template ships covers the desktop App:
    // `--locked` fails if resolving it would change the lock.
    for args in [
        ["build", "-p", DESKTOP_PACKAGE, "--locked"],
        ["test", "-p", DESKTOP_PACKAGE, "--locked"],
    ] {
        let status = cargo(root, &args).status()?;
        ensure!(
            status.success(),
            "`cargo {}` in {} failed: {status}",
            args.join(" "),
            common::shown(root)
        );
    }
    Ok(())
}
