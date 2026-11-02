//! The signed-in shell and the users screens, rendered by the App itself (its repositories,
//! blocs and routes) in the Rok theme's light and dark, against a mock API on localhost.

use desktop::app::App as DesktopApp;
use desktop::data::providers::api::Api;
use gpui::{TestAppContext, VisualTestContext};
use rok_ui::http::testing::MockServer;
use rok_ui::http::{Method, Session};
use rok_ui::prelude::*;
use rok_ui::router;
use serde_json::json;
use std::time::Duration;

const ID: &str = "0190c0de-0000-7000-8000-000000000002";

/// A user row, as `apps/api` answers it.
fn row(id: &str, email: &str, display_name: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "email": email,
        "display_name": display_name,
        "email_verified": true,
        "has_password": true,
        "is_active": true,
        "role": "user",
        "totp_enabled": false,
        "created_at": "2026-11-02T10:00:00Z",
    })
}

/// An API with two users, one page of them.
fn api() -> MockServer {
    let server = MockServer::start();
    server.on(Method::Get, "/api/v1/users").reply_json(
        200,
        &json!({
            "items": [
                row(ID, "grace@example.com", Some("Grace")),
                row("0190c0de-0000-7000-8000-000000000003", "ada@example.com", None),
            ],
            "page": 1,
            "per_page": 20,
            "total": 2,
        }),
    );
    server
        .on(Method::Get, &format!("/api/v1/users/{ID}"))
        .reply_json(200, &row(ID, "grace@example.com", Some("Grace")));
    server
}

/// The main window's view, as `main.rs` renders it.
struct Main {
    api: Api,
    session: Session,
}

impl Render for Main {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        AppRoot::new().child(DesktopApp::new(self.api.clone(), self.session.clone()))
    }
}

/// The App, signed in, against `server`, in the Rok theme's `mode`.
fn open<'a>(
    cx: &'a mut TestAppContext,
    server: &MockServer,
    mode: ThemeMode,
) -> &'a mut VisualTestContext {
    cx.update(rok_ui::init);
    cx.update(|cx| Theme::set_global(Theme::from_preset(ThemePreset::Rok, mode), cx));
    cx.executor().allow_parking();
    let session = Session::with_token("t0k3n");
    let api = Api::new(server.url(), session.clone());
    let (_, window) = cx.add_window_view(move |_, _| Main {
        api: api.clone(),
        session: session.clone(),
    });
    window.run_until_parked();
    window
}

/// Whether `selector` renders before long: the blocs answer on their own runtime.
fn shows(window: &mut VisualTestContext, selector: &'static str) -> bool {
    for _ in 0..200 {
        window.run_until_parked();
        if window.debug_bounds(selector).is_some() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn click(window: &mut VisualTestContext, selector: &'static str) {
    let bounds = window
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is not rendered"));
    window.simulate_click(bounds.center(), gpui::Modifiers::none());
    window.run_until_parked();
}

/// The list, then a click on a row, then that user's detail, all in `mode`.
fn the_users_screens_render_in(mode: ThemeMode, cx: &mut TestAppContext) {
    let server = api();
    let window = open(cx, &server, mode);
    window.update(|_, cx| router::navigate("/users", cx));

    assert!(shows(window, "users-table"), "the list renders");
    assert!(
        shows(window, "user-row-0190c0de-0000-7000-8000-000000000002"),
        "Grace's row renders"
    );
    assert_eq!(server.requests_to("/api/v1/users")[0].query, "page=1");

    click(window, "user-row-0190c0de-0000-7000-8000-000000000002");
    assert!(shows(window, "user-detail"), "Grace's detail renders");
    window.update(|_, cx| {
        assert_eq!(router::location(cx).path(), format!("/users/{ID}"));
        assert_eq!(cx.theme().preset, ThemePreset::Rok);
        assert_eq!(cx.theme().mode, mode);
    });
}

#[gpui::test]
fn the_users_screens_render_in_light(cx: &mut TestAppContext) {
    the_users_screens_render_in(ThemeMode::Light, cx);
}

#[gpui::test]
fn the_users_screens_render_in_dark(cx: &mut TestAppContext) {
    the_users_screens_render_in(ThemeMode::Dark, cx);
}

#[gpui::test]
fn the_app_bar_toggles_light_and_dark(cx: &mut TestAppContext) {
    let server = api();
    let window = open(cx, &server, ThemeMode::Light);
    window.update(|window, cx| desktop::shell::install_theme(window, cx));
    let start = window.update(|_, cx| cx.theme().mode);

    assert!(shows(window, "theme-toggle"), "the shell renders at /");
    click(window, "theme-toggle");
    window.update(|_, cx| {
        assert_eq!(cx.theme().mode, start.toggled());
        assert_eq!(
            cx.theme().preset,
            ThemePreset::Rok,
            "the toggle keeps the Rok theme"
        );
    });
    click(window, "theme-toggle");
    window.update(|_, cx| assert_eq!(cx.theme().mode, start));
}
