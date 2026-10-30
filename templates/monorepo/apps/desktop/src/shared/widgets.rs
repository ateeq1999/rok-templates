//! Widgets shared by features.

use rok_ui::bloc::BlocListener;
use rok_ui::http::Session;
use rok_ui::prelude::*;

/// Where the user signs in.
pub const SIGN_IN_PATH: &str = "/auth";

/// A page's frame: a title over the page's content.
#[component]
pub fn PageFrame(title: SharedString, #[children] children: Vec<AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(H2::new(title))
        .children(children)
}

/// Sends the user to [`SIGN_IN_PATH`] when the server expires their session (a 401), and
/// home once they sign in. Render it once, in the root layout.
#[component]
pub fn SessionGuard(cx: &mut Cx) -> impl IntoElement {
    let session = cx.repository::<Session>();
    BlocListener::new(&*session, |state, _, cx| {
        if state.expired {
            rok_ui::router::replace(SIGN_IN_PATH, cx);
        } else if state.token.is_some() {
            rok_ui::router::replace("/", cx);
        }
    })
    .listen_when(|previous, current| {
        previous.expired != current.expired || previous.token.is_some() != current.token.is_some()
    })
}
