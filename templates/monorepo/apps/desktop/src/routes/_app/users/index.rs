use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: UsersRoute,
}

/// `/users`: the users list. Routes stay thin; the page lives in `features::users::view`.
#[component]
fn UsersRoute() -> impl IntoElement {
    crate::features::users::view::users_page::UsersPage::new()
}
