// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The development seed. `cargo run --bin seed` makes a verified admin and
//! a verified ordinary user and prints how to sign in as each (a new
//! password every run); `cargo run --bin seed -- --promote <email>` makes
//! an existing account an admin. It reads the server's settings (from
//! `.env` too, in a debug build) and needs the migrations applied (`rok
//! migrate up`). The work is `app::seed`'s.

use std::io::Write;
use std::process::ExitCode;

use actix_api::app::seed;
use actix_api::config::{self, Config};
use actix_api::db::pool::init_pool;
use actix_api::db::types::Role;

/// How to call it.
const USAGE: &str = "usage: seed [--promote <email>]";

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Seeds, or promotes the account `--promote` names.
async fn run(args: &[String]) -> Result<(), String> {
    let promote = match args {
        [] => None,
        [flag, email] if flag == "--promote" => Some(email.as_str()),
        _ => return Err(USAGE.to_owned()),
    };
    config::load_env_file().map_err(|err| format!(".env: {err}"))?;
    let config = Config::from_env().map_err(|err| err.to_string())?;
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(|err| err.to_string())?;
    let mut out = std::io::stdout().lock();
    if let Some(email) = promote {
        let promoted = seed::promote(&pool, email)
            .await
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("no account has the email {email}"))?;
        return writeln!(out, "{promoted} is an admin now").map_err(|err| err.to_string());
    }
    let accounts = seed::seed(&pool).await.map_err(|err| err.to_string())?;
    writeln!(out, "Seeded two verified accounts; sign in with:").map_err(|err| err.to_string())?;
    let width = accounts
        .iter()
        .map(|account| account.email.len())
        .max()
        .unwrap_or(0);
    for account in accounts {
        let role = match account.role {
            Role::Admin => "admin",
            Role::User => "user",
        };
        let (email, password) = (&account.email, &account.password);
        writeln!(out, "  {role:<5}  {email:<width$}  {password}").map_err(|err| err.to_string())?;
    }
    Ok(())
}
