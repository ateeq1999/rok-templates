use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: NotFound,
}

/// Shown when no route matches.
#[component]
fn NotFound() -> impl IntoElement {
    Empty::new().title("Page not found")
}
