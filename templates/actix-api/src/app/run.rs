// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`run`]: the server, and the mail sender and the upload sweep beside it.

use actix_web::{App, HttpServer};

use super::{AppState, configure};
use crate::config::Config;
use crate::db::pool::init_pool;
use crate::shared::mail::{Sender, transport};

/// Logs the settings (secrets left out), connects the pool, starts the mail
/// sender, then serves the application on `config.bind_addr` until the
/// server stops, and stops the sender.
pub async fn run(config: Config) -> std::io::Result<()> {
    tracing::info!("settings:\n{}", config.summary());
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(std::io::Error::other)?;
    let mailer = transport::from_config(&config).map_err(std::io::Error::other)?;
    let state = AppState::new(pool, &config).map_err(std::io::Error::other)?;
    let sender = Sender::new(state.pool.clone(), mailer, &state.outbox);
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let sending = actix_web::rt::spawn(sender.run(async {
        // A dropped `stop` stops the sender too.
        let _ = stopped.await;
    }));
    if let Some(every) = state.media_upload_sweep {
        actix_web::rt::spawn(state.media_service().sweep_every(every));
    }
    tracing::info!(addr = %config.bind_addr, "listening");
    let server_state = state.clone();
    let served = HttpServer::new(move || App::new().configure(|cfg| configure(cfg, &server_state)))
        .bind(&config.bind_addr)?
        .run()
        .await;
    let _ = stop.send(());
    if let Err(err) = sending.await {
        tracing::error!(error = %err, "the mail sender failed");
    }
    served
}
