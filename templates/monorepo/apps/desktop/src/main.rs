//! Opens the main window. The app lives in `src/lib.rs`; its providers in `src/app.rs`.
//!
//! The API's address comes from `ROK_API_URL` (default [`API_URL`], `apps/api`'s `BIND_ADDR`).

use rok_ui::http::{HttpClient, Session};
use rok_ui::prelude::*;

/// Where `apps/api` listens by default (its `BIND_ADDR`), when `ROK_API_URL` is unset.
const API_URL: &str = "http://127.0.0.1:8080";

struct Main {
    client: HttpClient,
    session: Session,
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppRoot::new().child(desktop::app::App::new(
            self.client.clone(),
            self.session.clone(),
        ))
    }
}

fn main() {
    // The session holds the token; the client sends it, and expires the session on a 401.
    // The token lives in memory: see `Session::persisted` to keep users signed in.
    let session = Session::new();
    let base_url = std::env::var("ROK_API_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| API_URL.to_owned());
    let client = HttpClient::builder()
        .base_url(base_url)
        .session(session.clone())
        .build();
    Application::new()
        .with_assets(rok_ui::Assets)
        .run(move |cx: &mut gpui::App| {
            rok_ui::init(cx);
            cx.open_window(WindowOptions::default(), |_, cx| {
                cx.new(|_| Main { client, session })
            })
            .expect("the window opens");
        });
}
