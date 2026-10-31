//! Writes the `OpenAPI` document this API serves at `/api-docs/openapi.json`
//! to the file its one argument names, as pretty-printed JSON: `rok gen`
//! runs it from the monorepo's root to re-export `openapi.json`, the
//! contract the Clients under `clients/` are generated from (workspace ADR
//! 0027). `tests/openapi_artifact.rs` keeps the exported file and the served
//! document the same.
//!
//! ```sh
//! cargo run -p axum-api --example openapi -- apps/api/openapi.json
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: openapi <PATH>: the file the document is written to");
        return ExitCode::FAILURE;
    };
    let json = match axum_api::openapi::document().to_pretty_json() {
        Ok(json) => json,
        Err(err) => {
            eprintln!("error: the OpenAPI document does not serialize: {err}");
            return ExitCode::FAILURE;
        }
    };
    match std::fs::write(&path, format!("{json}\n")) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: writing {}: {err}", path.to_string_lossy());
            ExitCode::FAILURE
        }
    }
}
