use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: Home,
}

/// `/`: the start page, where signing in lands.
#[component]
fn Home() -> impl IntoElement {
    crate::shared::widgets::PageFrame::new("Welcome")
        .child(P::new(
            "You are signed in. The users the API holds are a click away.",
        ))
        .child(
            div().child(
                Button::new("open-users")
                    .label("Browse users")
                    .on_click(|_, _, cx| rok_ui::router::navigate("/users", cx)),
            ),
        )
}
