//! Opens the main window. The app lives in `src/lib.rs`; its providers in `src/app.rs`.
//!
//! The API's address comes from `ROK_API_URL` (default [`API_URL`], `apps/api`'s `BIND_ADDR`).

use std::sync::Arc;

use desktop::data::providers::api::Api;
use desktop::data::providers::token_store::KeychainTokenStore;
use rok_ui::prelude::*;

/// Where `apps/api` listens by default (its `BIND_ADDR`), when `ROK_API_URL` is unset.
const API_URL: &str = "http://127.0.0.1:8080";

struct Main {
    api: Api,
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppRoot::new().child(desktop::app::App::new(self.api.clone()))
    }
}

fn main() {
    let base_url = std::env::var("ROK_API_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| API_URL.to_owned());
    // The session holds the token; the API's client sends it, and a 401 expires the session.
    // The OS keychain keeps it between launches, one entry per API address: signing in saves
    // it, and signing out or a 401 deletes it.
    let store = Arc::new(KeychainTokenStore::new(base_url.clone()));
    let api = Api::kept(base_url, store);
    Application::new()
        .with_assets(rok_ui::Assets)
        .run(move |cx: &mut gpui::App| {
            rok_ui::init(cx);
            let opened = cx.open_window(WindowOptions::default(), |window, cx| {
                desktop::shell::install_theme(window, cx);
                cx.new(|_| Main { api })
            });
            if let Err(error) = opened {
                eprintln!("the main window did not open: {error:#}");
                std::process::exit(1);
            }
        });
}
