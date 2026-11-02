use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: Home,
}

/// `/`: the start page, where signing in lands.
#[component]
fn Home() -> impl IntoElement {
    crate::shared::widgets::PageFrame::new("Welcome")
        .child(P::new("You are signed in. Your account is a click away."))
        .child(
            div().debug_selector(|| "open-account".into()).child(
                Button::new("open-account")
                    .label("My account")
                    .on_click(|_, _, cx| rok_ui::router::navigate("/account", cx)),
            ),
        )
}
