use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    layout: Shell,
}

/// The frame around every page: navigation, then the page. The session guard sends the
/// user to sign in when the server expires their session.
#[component]
fn Shell(#[children] children: Vec<AnyElement>) -> impl IntoElement {
    let links = [
        Link::to(&super::Index).exact(true).child("Home"),
        Link::new("nav-notes", "/notes").child("Notes"),
        Link::new("nav-auth", "/auth").child("Auth"),
        // rok-ui:nav
    ];
    div()
        .flex()
        .flex_col()
        .size_full()
        .child(crate::shared::widgets::SessionGuard::new())
        .child(div().flex().gap_4().p_4().border_b_1().children(links))
        .child(div().flex_1().children(children))
}
