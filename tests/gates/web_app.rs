//! The `apps/web` App of the monorepo template, made the way a user makes
//! it: `rok init shop --template monorepo` into a scratch directory, then, in
//! the scaffolded `apps/web`, the web App's own checks with its own
//! tooling — `bun install --frozen-lockfile` (the template's `bun.lock`
//! must match its `package.json`), `bun run routes` (which must leave the
//! shipped route tree as it is), `bun run typecheck` and `bun run test`
//! (the App's vitest suite; `bun test`, bun's own runner, is not it), and
//! `bun run dev` answers at `127.0.0.1:3001`.
//! The App calls the API through the generated `clients/ts`, which init
//! emits beside it; a contract change is followed through `rok gen` and
//! the same checks. `bun run e2e`, the browser smoke flow, runs against the
//! scaffolded API on a database of its own. Skips with a printed note when
//! bun is absent, and the smoke flow when no database is given to run the
//! API on.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};

use crate::common::{self, cargo};
use crate::monorepo_template::API_PACKAGE;

/// The `bun` to run: the one on `PATH`, else the one bun's installer puts
/// under `~/.bun/bin`; `None` when neither runs.
fn bun() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| {
            PathBuf::from(home).join(format!(".bun/bin/bun{}", std::env::consts::EXE_SUFFIX))
        });
    std::iter::once(PathBuf::from("bun"))
        .chain(home)
        .find(|bun| {
            Command::new(bun)
                .arg("--version")
                .output()
                .is_ok_and(|output| output.status.success())
        })
}

/// `bun <args>` in `dir`, which must succeed. No Node flag is added: the
/// App's `test` script turns Node's own Web Storage off itself, so the gate
/// runs it as a developer does, on any Node.
fn run(bun: &Path, dir: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new(bun)
        .args(args)
        .current_dir(dir)
        .output()
        .context("failed to run bun")?;
    ensure!(
        output.status.success(),
        "bun {args:?} in {} failed:\n{}{}",
        common::shown(dir),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// The scaffold writes the web App's binary files byte for byte, and its
/// `.env.example`.
#[test]
fn monorepo_scaffold_writes_the_web_app_whole() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let project = crate::monorepo_template::scaffold()?;
    let template = common::template_dir("monorepo").join("apps/web");
    let web = project.path().join("apps/web");
    for file in ["public/favicon.ico", "public/logo.png"] {
        ensure!(
            std::fs::read(web.join(file))? == std::fs::read(template.join(file))?,
            "the scaffolded apps/web/{file} differs from the template's"
        );
    }
    for file in [
        ".env.example",
        "package.json",
        "bun.lock",
        "vitest.config.ts",
    ] {
        ensure!(
            web.join(file).is_file(),
            "the scaffold has no apps/web/{file}"
        );
    }
    Ok(())
}

/// The gate: scaffold the monorepo, then install the web App's locked
/// dependencies, typecheck it, run its tests and start its dev server.
#[test]
fn monorepo_web_app_installs_typechecks_and_tests() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(bun) = bun() else {
        eprintln!(
            "note: skipping the web App gate: `bun` is not installed (https://bun.sh installs it)"
        );
        return Ok(());
    };
    let project = crate::monorepo_template::scaffold()?;
    let web = project.path().join("apps/web");
    run(&bun, &web, &["install", "--frozen-lockfile"])?;
    route_tree_is_current(&bun, &web)?;
    run(&bun, &web, &["run", "typecheck"])?;
    run(&bun, &web, &["run", "test"])?;
    dev_server_answers_on_ipv4(&bun, &web)?;
    Ok(())
}

/// `bun run routes` (the router's own `tsr generate`, no build) leaves the
/// shipped `routeTree.gen.ts` as it is: it writes the tree Vite's router
/// plugin writes, so a developer adds a route without starting Vite.
fn route_tree_is_current(bun: &Path, web: &Path) -> Result<()> {
    let tree = web.join("src/routeTree.gen.ts");
    let shipped =
        std::fs::read_to_string(&tree).context("the web App has no src/routeTree.gen.ts")?;
    run(bun, web, &["run", "routes"])?;
    let generated = std::fs::read_to_string(&tree)?;
    if let Some(line) = first_difference(&shipped, &generated) {
        anyhow::bail!(
            "`bun run routes` changed apps/web/src/routeTree.gen.ts at line {line}: the \
             shipped tree is stale (regenerate it with `bun run routes`), or tsr.config.json's \
             `routeTreeFileFooter` no longer matches TanStack Start's"
        );
    }
    Ok(())
}

/// The first line (from 1) at which `a` and `b` differ, if they do.
fn first_difference(a: &str, b: &str) -> Option<usize> {
    if a == b {
        return None;
    }
    let (mut a_lines, mut b_lines) = (a.lines(), b.lines());
    let mut line = 1;
    loop {
        match (a_lines.next(), b_lines.next()) {
            (Some(x), Some(y)) if x == y => line += 1,
            _ => return Some(line),
        }
    }
}
/// The port `bun run dev` serves on (`package.json`).
const DEV_PORT: u16 = 3001;

/// `bun run dev` answers `GET /` at `127.0.0.1:3001`, the address scripts
/// use on every OS: Vite's default host, `localhost`, is `::1` alone on
/// some systems (Windows among them).
fn dev_server_answers_on_ipv4(bun: &Path, web: &Path) -> Result<()> {
    let address = dev_address()?;
    let server = Spawned::dev_server(bun, web)?;
    let response = wait_for(address, "/", Duration::from_secs(60));
    drop(server);
    let response = response.map_err(|error| {
        anyhow::anyhow!(
            "`bun run dev` did not answer at http://{address} within 60s ({error}): \
             vite.config.ts's `server.host` should be \"127.0.0.1\""
        )
    })?;
    ensure!(
        response.starts_with("HTTP/1.1 200"),
        "`bun run dev` answered GET / at http://{address} with:\n{}",
        response.lines().next().unwrap_or_default()
    );
    Ok(())
}

/// `127.0.0.1:3001`, once checked to be free: a server already on the port
/// would answer for the one under test.
fn dev_address() -> Result<SocketAddr> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, DEV_PORT));
    drop(TcpListener::bind(address).with_context(|| {
        format!("{address} is taken: stop what listens there to run the web gate")
    })?);
    Ok(address)
}

/// `GET path` at `address`, retried until it answers or `patience` runs
/// out; the whole response.
fn wait_for(address: SocketAddr, path: &str, patience: Duration) -> std::io::Result<String> {
    let deadline = Instant::now() + patience;
    loop {
        match get(address, path) {
            Ok(response) => return Ok(response),
            Err(error) if Instant::now() >= deadline => return Err(error),
            Err(_) => std::thread::sleep(Duration::from_millis(500)),
        }
    }
}

/// One `GET path` at `address`; the whole response.
fn get(address: SocketAddr, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.write_all(
        format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").as_bytes(),
    )?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    Ok(String::from_utf8_lossy(&response).into_owned())
}

/// A process the gate started, stopped with the processes it started in
/// turn (Vite under `bun run dev`, the server under `cargo run`) when
/// dropped.
struct Spawned(std::process::Child);

impl Spawned {
    /// Starts `command`, its output discarded.
    fn start(mut command: Command, what: &str) -> Result<Self> {
        command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut command, 0);
        let child = command
            .spawn()
            .with_context(|| format!("failed to start {what}"))?;
        Ok(Self(child))
    }

    /// `bun run dev` in `web`.
    fn dev_server(bun: &Path, web: &Path) -> Result<Self> {
        let mut command = Command::new(bun);
        command.args(["run", "dev"]).current_dir(web);
        Self::start(command, "`bun run dev`")
    }
}

impl Drop for Spawned {
    fn drop(&mut self) {
        let pid = self.0.id().to_string();
        // Killing bun or cargo alone would leave its child holding the port.
        #[cfg(windows)]
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid])
            .output();
        #[cfg(unix)]
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{pid}")])
            .output();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The browser smoke flow: `bun run e2e` (Playwright) signs in with the dev
/// seed's account, lands on home and signs out, against the scaffolded API
/// running on a database of its own. The gate starts the API, so it needs
/// PostgreSQL: without `ROK_DATA_SQLC_TEST_DATABASE_URL` no API is reachable
/// and it skips with a note.
#[test]
fn monorepo_web_app_signs_in_and_out_in_a_browser() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(bun) = bun() else {
        eprintln!(
            "note: skipping the browser smoke flow: `bun` is not installed (https://bun.sh installs it)"
        );
        return Ok(());
    };
    let Ok(server) = std::env::var("ROK_DATA_SQLC_TEST_DATABASE_URL") else {
        eprintln!(
            "note: skipping the browser smoke flow: no API is reachable to run it against \
             (set ROK_DATA_SQLC_TEST_DATABASE_URL and the gate starts one)"
        );
        return Ok(());
    };
    let dev = dev_address()?;
    let project = crate::monorepo_template::materialize()?;
    let dir = project.path();
    let database = Database::create(dir, &server)?;
    rok(
        dir,
        &[
            "migrate",
            "up",
            "--dir",
            "apps/api/migrations",
            "--database-url",
            &database.url,
        ],
    )?;

    // The API's settings, which the seed `bun run e2e` runs reads too.
    let api = {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.local_addr()?
    };
    let settings = [
        ("DATABASE_URL", database.url.clone()),
        (
            "JWT_SECRET",
            "a-gate-only-secret-of-at-least-32-bytes".to_owned(),
        ),
        (
            "TOTP_ENCRYPTION_KEY",
            "MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDE=".to_owned(),
        ),
        ("BIND_ADDR", api.to_string()),
        ("APP_ROLE", "web".to_owned()),
        ("AUTH_RATE_LIMIT", "0".to_owned()),
    ];
    let build = cargo(dir, &["build", "-p", API_PACKAGE, "--bins"]).status()?;
    ensure!(
        build.success(),
        "`cargo build -p {API_PACKAGE}` failed: {build}"
    );
    let mut command = cargo(dir, &["run", "-p", API_PACKAGE]);
    command.envs(settings.clone());
    let server = Spawned::start(command, "the API")?;
    let health = wait_for(api, "/health/live", Duration::from_secs(120))
        .with_context(|| format!("the API did not answer at http://{api} within 120s"))?;
    ensure!(
        health.starts_with("HTTP/1.1 200"),
        "the API answered GET /health/live with:\n{health}"
    );

    let web = dir.join("apps/web");
    run(&bun, &web, &["install", "--frozen-lockfile"])?;
    run(&bun, &web, &["x", "playwright", "install", "chromium"])?;
    // `CI` turns an unreachable API into a failure instead of a skip, and
    // starts a dev server of the run's own; the seed `bun run e2e` runs
    // builds in the gates' target directory.
    let mut e2e = Command::new(&bun);
    e2e.args(["run", "e2e"])
        .current_dir(&web)
        .envs(settings)
        .env("API_PROXY_TARGET", format!("http://{api}"))
        .env("CI", "1");
    for (key, value) in cargo(dir, &[]).get_envs() {
        if let Some(value) = value {
            e2e.env(key, value);
        }
    }
    let output = e2e.output().context("failed to run `bun run e2e`")?;
    drop(server);
    ensure!(
        output.status.success(),
        "`bun run e2e` against the API at http://{api} (the App at http://{dev}) failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    drop(database);
    Ok(())
}

/// `rok <args>` in `dir`, which must succeed.
fn rok(dir: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new(crate::rok_data_bin()?)
        .args(args)
        .current_dir(dir)
        .output()
        .context("failed to run rok")?;
    ensure!(
        output.status.success(),
        "rok {args:?} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// A database of the gate's own on the test server (`rok db create`),
/// dropped with it (`rok db drop`).
struct Database {
    dir: PathBuf,
    server: String,
    name: String,
    url: String,
}

impl Database {
    fn create(dir: &Path, server: &str) -> Result<Self> {
        let name = format!("web_e2e_{}", std::process::id());
        // A database left by an earlier run that stopped half way.
        let _ = rok(
            dir,
            &["db", "drop", &name, "--database-url", server, "--yes"],
        );
        rok(dir, &["db", "create", &name, "--database-url", server])?;
        Ok(Self {
            dir: dir.to_owned(),
            server: server.to_owned(),
            url: with_database(server, &name),
            name,
        })
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let _ = rok(
            &self.dir,
            &[
                "db",
                "drop",
                &self.name,
                "--database-url",
                &self.server,
                "--yes",
            ],
        );
    }
}

/// `url` with its database replaced by `name`.
fn with_database(url: &str, name: &str) -> String {
    let (base, query) = url
        .split_once('?')
        .map_or((url, None), |(b, q)| (b, Some(q)));
    let authority_end = base.find("://").map_or(0, |at| at + 3);
    let path = base[authority_end..]
        .find('/')
        .map_or(base.len(), |at| authority_end + at);
    let mut out = format!("{}/{name}", &base[..path]);
    if let Some(query) = query {
        out.push('?');
        out.push_str(query);
    }
    out
}

#[test]
fn with_database_replaces_only_the_database() {
    assert_eq!(
        with_database("postgres://u:p@localhost:5432/postgres", "e2e"),
        "postgres://u:p@localhost:5432/e2e"
    );
    assert_eq!(
        with_database("postgres://u:p@localhost/app?sslmode=disable", "e2e"),
        "postgres://u:p@localhost/e2e?sslmode=disable"
    );
    assert_eq!(
        with_database("postgres://localhost", "e2e"),
        "postgres://localhost/e2e"
    );
}

/// A contract change round-trips through regeneration and the gate: a
/// change to the api App's source is re-exported by `rok gen` into
/// `openapi.json`, regenerates `clients/ts`, leaves `rok gen check` clean,
/// and the web App installs, typechecks and passes its tests with the new
/// client (workspace ADR 0027).
#[test]
fn monorepo_contract_change_round_trips_through_the_web_gate() -> Result<()> {
    if crate::generator_missing() {
        return Ok(());
    }
    let Some(bun) = bun() else {
        eprintln!(
            "note: skipping the contract round trip: `bun` is not installed (https://bun.sh installs it)"
        );
        return Ok(());
    };
    let project = crate::monorepo_template::materialize()?;
    let dir = project.path();
    let before = std::fs::read_to_string(dir.join("clients/ts/src/types.ts"))?;

    // The change: a field of the API's `TotpSetupResponse` documented anew.
    // The secret's description is the only one of its words in the api
    // App's sources and in its document.
    const OLD: &str = "Base32-encoded; lets the user type the secret in by hand if they\n    /// cannot scan the QR code.";
    const NEW: &str = "The secret, base32-encoded, for typing in by hand.";
    let dto = dir.join("apps/api/src/features/two_factor/dto.rs");
    let source = std::fs::read_to_string(&dto)?.replace("\r\n", "\n");
    ensure!(
        source.contains(OLD),
        "the api App's two-factor DTO no longer says `{OLD}`: pick another change"
    );
    std::fs::write(&dto, source.replace(OLD, NEW))?;

    crate::monorepo_template::rok_gen_shared(dir, &[])?;
    let contract = std::fs::read_to_string(dir.join("apps/api/openapi.json"))?;
    ensure!(
        contract.contains(NEW),
        "`rok gen` did not re-export the changed contract"
    );
    let after = std::fs::read_to_string(dir.join("clients/ts/src/types.ts"))?;
    ensure!(
        after != before && after.contains(NEW),
        "`rok gen` did not regenerate clients/ts from the changed contract"
    );
    crate::monorepo_template::rok_gen_shared(dir, &["check"])?;

    let web = dir.join("apps/web");
    run(&bun, &web, &["install", "--frozen-lockfile"])?;
    run(&bun, &web, &["run", "typecheck"])?;
    run(&bun, &web, &["run", "test"])?;
    Ok(())
}
