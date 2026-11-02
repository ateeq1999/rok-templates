use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    layout: AppLayout,
}

/// The signed-in pages (the files under `_app/`, which adds no path segment) inside the shell.
#[component]
fn AppLayout(#[children] children: Vec<AnyElement>) -> impl IntoElement {
    crate::shell::AppShell::new().children(children)
}
