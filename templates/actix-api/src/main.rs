// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Starts the server. Nothing else: the modules are declared in `lib.rs`,
//! wiring lives in `app/`, settings in `config.rs`.

use std::process::ExitCode;

use actix_api::shared::telemetry;
use actix_api::{app, config};

#[actix_web::main]
async fn main() -> ExitCode {
    // Logs and traces first (`RUST_LOG`, `LOG_FORMAT`, `OTEL_*`), so
    // everything after is traced.
    let telemetry = match config::log_settings()
        .map_err(|err| err.to_string())
        .and_then(|settings| telemetry::init(&settings).map_err(|err| err.to_string()))
    {
        Ok(telemetry) => telemetry,
        Err(err) => return failed(&err),
    };
    let started = match config::Config::from_env() {
        Ok(config) => app::run(config).await.map_err(|err| err.to_string()),
        Err(err) => Err(err.to_string()),
    };
    // The spans still buffered are sent before the process ends.
    telemetry.shutdown();
    match started {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => failed(&err),
    }
}

/// One readable line per wrong setting, such as "error: environment variable
/// `JWT_SECRET` is not set", and a non-zero exit.
fn failed(err: &str) -> ExitCode {
    eprintln!("error: {err}");
    ExitCode::FAILURE
}
