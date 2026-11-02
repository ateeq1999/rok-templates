//! Opens the main window. The app lives in `src/lib.rs`; its providers in `src/app.rs`.
//!
//! The API's address comes from `ROK_API_URL` (default [`API_URL`], `apps/api`'s `BIND_ADDR`).

use desktop::data::providers::api::Api;
use rok_ui::http::Session;
use rok_ui::prelude::*;

/// Where `apps/api` listens by default (its `BIND_ADDR`), when `ROK_API_URL` is unset.
const API_URL: &str = "http://127.0.0.1:8080";

struct Main {
    api: Api,
    session: Session,
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppRoot::new().child(desktop::app::App::new(
            self.api.clone(),
            self.session.clone(),
        ))
    }
}

fn main() {
    // The session holds the token; the API's client sends it, and a 401 expires the session.
    // The token lives in memory: see `Session::persisted` to keep users signed in.
    let session = Session::new();
    let base_url = std::env::var("ROK_API_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| API_URL.to_owned());
    let api = Api::new(base_url, session.clone());
    Application::new()
        .with_assets(rok_ui::Assets)
        .run(move |cx: &mut gpui::App| {
            rok_ui::init(cx);
            cx.open_window(WindowOptions::default(), |_, cx| {
                cx.new(|_| Main { api, session })
            })
            .expect("the window opens");
        });
}
