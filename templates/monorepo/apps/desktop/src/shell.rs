//! The signed-in shell: rok-ui's `AdaptiveScaffold` around every signed-in page, its
//! navigation following the window's width (a bottom bar, a rail, a rail with labels), and an
//! app bar with the light/dark toggle and sign-out. The Rok theme is installed in `main.rs`
//! ([`install_theme`]).

use crate::data::models::user::User;
use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_event::AuthEvent;
use crate::shared::widgets::SIGN_IN_PATH;
use rok_ui::http::Session;
use rok_ui::prelude::*;
use rok_ui::router;

/// A place the navigation goes. A path other than `/` is selected on its own pages and every
/// page under it (`/users` on `/users/<id>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Destination {
    /// Where it goes.
    pub path: &'static str,
    /// Its label, and the app bar's title on its pages.
    pub label: &'static str,
    /// Its icon.
    pub icon: IconName,
    /// Whether only an admin sees it: its pages are `apps/api`'s admin routes.
    pub admin_only: bool,
}

/// Every destination, in the navigation's order; [`destinations`] picks a user's.
pub const DESTINATIONS: [Destination; 3] = [
    Destination {
        path: "/",
        label: "Home",
        icon: IconName::Home,
        admin_only: false,
    },
    Destination {
        path: "/account",
        label: "My account",
        icon: IconName::User,
        admin_only: false,
    },
    Destination {
        path: "/users",
        label: "Users",
        icon: IconName::Inbox,
        admin_only: true,
    },
];

/// The destinations `user` sees: everyone their own account, an admin the users too.
#[must_use]
pub fn destinations(user: Option<&User>) -> Vec<Destination> {
    let admin = user.is_some_and(User::is_admin);
    DESTINATIONS
        .into_iter()
        .filter(|destination| admin || !destination.admin_only)
        .collect()
}

/// The Rok theme, in the light or dark the operating system shows. The app bar toggles it.
pub fn install_theme(window: &Window, cx: &mut App) {
    Theme::set_global(Theme::from_preset(ThemePreset::Rok, ThemeMode::Light), cx);
    Theme::sync_with_system_appearance(window, cx);
}

/// The destination the current location is in, as an index into `destinations`.
fn selected(destinations: &[Destination], cx: &mut App) -> usize {
    destinations
        .iter()
        .rposition(|destination| router::is_active(destination.path, destination.path == "/", cx))
        .unwrap_or(0)
}

/// The shell around a signed-in page (`children`). Signed out (at a start with no kept token,
/// or once the session is gone), it shows nothing and sends the user to sign in.
#[component]
pub fn AppShell(#[children] children: Vec<AnyElement>, cx: &mut Cx) -> impl IntoElement {
    if !cx.repository::<Session>().is_signed_in() {
        // After this frame: the router's location cannot change while it renders.
        cx.window
            .defer(cx.app, |_, cx| router::replace(SIGN_IN_PATH, cx));
        return div().into_any_element();
    }
    let destinations = destinations(cx.watch_bloc::<AuthBloc>().user.as_ref());
    let selected = selected(&destinations, cx);
    let title = destinations
        .get(selected)
        .map_or("", |destination| destination.label);
    let dark = cx.theme().mode.is_dark();
    let auth = cx.bloc::<AuthBloc>();
    let theme_toggle = div().debug_selector(|| "theme-toggle".into()).child(
        Button::new("theme-toggle")
            .ghost()
            .icon_only(if dark { IconName::Sun } else { IconName::Moon })
            .tooltip(if dark {
                "Switch to light"
            } else {
                "Switch to dark"
            })
            .on_click(|_, _, cx| Theme::toggle_mode(cx)),
    );
    let sign_out = div().debug_selector(|| "sign-out".into()).child(
        Button::new("sign-out")
            .ghost()
            .icon_only(IconName::LogOut)
            .tooltip("Sign out")
            .on_click(move |_, _, _| auth.add(AuthEvent::SignOutRequested)),
    );
    let items = destinations
        .iter()
        .map(|destination| NavigationDestination::new(destination.icon, destination.label));
    let scaffold = AdaptiveScaffold::new("shell")
        .app_bar(
            AppBar::new()
                .title(title)
                .action(theme_toggle)
                .action(sign_out),
        )
        .destinations(items)
        .selected_index(selected);
    scaffold
        .on_change(move |index, _, cx| {
            if let Some(destination) = destinations.get(*index) {
                router::navigate(destination.path, cx);
            }
        })
        .children(children)
        .into_any_element()
}
