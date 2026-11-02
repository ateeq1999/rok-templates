use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    layout: Root,
}

/// The frame around every page. The signed-in pages bring their own shell (`_app.rs`); the
/// session guard sends the user to sign in when the server expires their session.
#[component]
fn Root(#[children] children: Vec<AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .child(crate::shared::widgets::SessionGuard::new())
        .child(div().flex_1().size_full().children(children))
}
