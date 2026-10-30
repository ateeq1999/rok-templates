use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: NotesRoute,
}

/// `/notes`: the notes page. Routes stay thin; the page lives in `features::notes::view`.
#[component]
fn NotesRoute() -> impl IntoElement {
    crate::features::notes::view::notes_page::NotesPage::new()
}
