use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: Home,
}

/// The start page.
#[component]
fn Home() -> impl IntoElement {
    div().p_6().child(H1::new("desktop"))
}
