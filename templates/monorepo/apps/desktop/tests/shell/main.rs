//! The signed-in shell, the account and the users screens, rendered by the App itself (its
//! repositories, blocs and routes) in the Rok theme's light and dark, against a mock API on
//! localhost: every user reaches their account, an admin the users too.

use desktop::app::App as DesktopApp;
use desktop::data::models::user::{Role, User};
use desktop::data::providers::api::Api;
use desktop::data::providers::token_store::MemoryTokenStore;
use gpui::{TestAppContext, VisualTestContext};
use rok_ui::http::testing::MockServer;
use rok_ui::http::{Method, Session};
use rok_ui::prelude::*;
use rok_ui::router;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

const ID: &str = "0190c0de-0000-7000-8000-000000000002";

/// A user row, as `apps/api` answers it.
fn row(id: &str, email: &str, display_name: Option<&str>, role: &str) -> serde_json::Value {
    json!({
        "id": id,
        "email": email,
        "display_name": display_name,
        "email_verified": true,
        "has_password": true,
        "is_active": true,
        "role": role,
        "totp_enabled": false,
        "created_at": "2026-11-02T10:00:00Z",
    })
}

/// An API with two users, one page of them, signed in as Grace, who holds `role`.
fn api(role: &str) -> MockServer {
    let server = MockServer::start();
    server.on(Method::Get, "/api/v1/users").reply_json(
        200,
        &json!({
            "items": [
                row(ID, "grace@example.com", Some("Grace"), role),
                row("0190c0de-0000-7000-8000-000000000003", "ada@example.com", None, "user"),
            ],
            "page": 1,
            "per_page": 20,
            "total": 2,
        }),
    );
    server
        .on(Method::Get, &format!("/api/v1/users/{ID}"))
        .reply_json(200, &row(ID, "grace@example.com", Some("Grace"), role));
    // The kept token's account, which the App asks for at start.
    server
        .on(Method::Get, "/api/v1/auth/me")
        .reply_json(200, &row(ID, "grace@example.com", Some("Grace"), role));
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

/// The App, signed in with a token kept from an earlier launch, against `server`, in the Rok
/// theme's `mode`.
fn open<'a>(
    cx: &'a mut TestAppContext,
    server: &MockServer,
    mode: ThemeMode,
) -> &'a mut VisualTestContext {
    open_with(cx, server, mode, MemoryTokenStore::with_token("t0k3n"))
}

/// The App as `main.rs` builds it, its token kept in `store` (in memory: never the keychain).
fn open_with<'a>(
    cx: &'a mut TestAppContext,
    server: &MockServer,
    mode: ThemeMode,
    store: MemoryTokenStore,
) -> &'a mut VisualTestContext {
    cx.update(rok_ui::init);
    cx.update(|cx| Theme::set_global(Theme::from_preset(ThemePreset::Rok, mode), cx));
    cx.executor().allow_parking();
    let api = Api::kept(server.url(), Arc::new(store));
    let session = api.session().clone();
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
    let server = api("admin");
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
    let server = api("user");
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

/// Whether the location reaches `path` before long.
fn lands_on(window: &mut VisualTestContext, path: &str) -> bool {
    for _ in 0..200 {
        window.run_until_parked();
        if window.update(|_, cx| router::location(cx).path() == path) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

#[gpui::test]
fn a_signed_out_start_lands_on_sign_in(cx: &mut TestAppContext) {
    let server = api("user");
    let window = open_with(cx, &server, ThemeMode::Light, MemoryTokenStore::new());

    assert!(lands_on(window, "/auth"), "no kept token: sign in first");
    assert!(
        window.debug_bounds("theme-toggle").is_none(),
        "the shell stays hidden"
    );
}

#[gpui::test]
fn signing_out_from_the_shell_forgets_the_kept_token(cx: &mut TestAppContext) {
    let server = api("user");
    let store = MemoryTokenStore::with_token("t0k3n");
    let window = open_with(cx, &server, ThemeMode::Light, store.clone());

    assert!(shows(window, "sign-out"), "the shell renders, signed in");
    click(window, "sign-out");

    assert!(lands_on(window, "/auth"), "signing out goes to sign in");
    assert_eq!(store.token(), None, "the kept token is gone");
}

/// The labels of the navigation the shell shows `user`.
fn navigation_for(role: Role) -> Vec<&'static str> {
    let user = User {
        role,
        ..User::default()
    };
    desktop::shell::destinations(Some(&user))
        .iter()
        .map(|destination| destination.label)
        .collect()
}

#[test]
fn a_user_is_shown_their_account_and_no_users() {
    assert_eq!(navigation_for(Role::User), ["Home", "My account"]);
}

#[test]
fn an_admin_is_shown_their_account_and_the_users() {
    assert_eq!(navigation_for(Role::Admin), ["Home", "My account", "Users"]);
}

#[test]
fn nobody_is_shown_the_users_before_their_account_answers() {
    assert_eq!(
        desktop::shell::destinations(None)
            .iter()
            .map(|destination| destination.label)
            .collect::<Vec<_>>(),
        ["Home", "My account"]
    );
}

#[gpui::test]
fn home_opens_the_signed_in_account(cx: &mut TestAppContext) {
    let server = api("user");
    let window = open(cx, &server, ThemeMode::Light);

    assert!(shows(window, "open-account"), "home renders");
    click(window, "open-account");

    assert!(lands_on(window, "/account"), "home points to the account");
    assert!(shows(window, "account"), "Grace's account renders");
}

#[gpui::test]
fn a_user_opening_the_users_is_told_they_need_the_admin_role(cx: &mut TestAppContext) {
    let server = api("user");
    let window = open(cx, &server, ThemeMode::Light);
    window.update(|_, cx| router::navigate("/users", cx));

    assert!(shows(window, "admin-required"), "the screen says why");
    assert!(
        window.debug_bounds("users-table").is_none(),
        "no list for a user"
    );
    assert!(
        server.requests_to("/api/v1/users").is_empty(),
        "nothing asks the API for its admin routes"
    );
}

#[gpui::test]
fn an_admin_reaches_their_account_and_the_users(cx: &mut TestAppContext) {
    let server = api("admin");
    let window = open(cx, &server, ThemeMode::Light);

    window.update(|_, cx| router::navigate("/account", cx));
    assert!(shows(window, "account"), "the account renders");
    window.update(|_, cx| router::navigate("/users", cx));
    assert!(shows(window, "users-table"), "the list renders");
}
