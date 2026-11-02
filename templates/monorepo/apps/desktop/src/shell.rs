//! The signed-in shell: rok-ui's `AdaptiveScaffold` around every signed-in page, its
//! navigation following the window's width (a bottom bar, a rail, a rail with labels), and an
//! app bar with the light/dark toggle and sign-out. The Rok theme is installed in `main.rs`
//! ([`install_theme`]).

use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_event::AuthEvent;
use crate::shared::widgets::SIGN_IN_PATH;
use rok_ui::http::Session;
use rok_ui::prelude::*;
use rok_ui::router;

/// Where the navigation goes: a path, its label and its icon. A path other than `/` is
/// selected on its own pages and every page under it (`/users` on `/users/<id>`).
pub const DESTINATIONS: [(&str, &str, IconName); 2] = [
    ("/", "Home", IconName::Home),
    ("/users", "Users", IconName::User),
];

/// The Rok theme, in the light or dark the operating system shows. The app bar toggles it.
pub fn install_theme(window: &Window, cx: &mut App) {
    Theme::set_global(Theme::from_preset(ThemePreset::Rok, ThemeMode::Light), cx);
    Theme::sync_with_system_appearance(window, cx);
}

/// The destination the current location is in, as an index into [`DESTINATIONS`].
fn selected(cx: &mut App) -> usize {
    DESTINATIONS
        .iter()
        .rposition(|(path, _, _)| router::is_active(path, *path == "/", cx))
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
    let selected = selected(cx);
    let title = DESTINATIONS
        .get(selected)
        .map_or("", |(_, label, _)| *label);
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
    AdaptiveScaffold::new("shell")
        .app_bar(
            AppBar::new()
                .title(title)
                .action(theme_toggle)
                .action(sign_out),
        )
        .destinations(DESTINATIONS.map(|(_, label, icon)| NavigationDestination::new(icon, label)))
        .selected_index(selected)
        .on_change(|index, _, cx| {
            if let Some((path, _, _)) = DESTINATIONS.get(*index) {
                router::navigate(*path, cx);
            }
        })
        .children(children)
        .into_any_element()
}
