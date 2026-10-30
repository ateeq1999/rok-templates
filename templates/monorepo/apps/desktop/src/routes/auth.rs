use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: AuthRoute,
}

/// `/auth`: the auth page. Routes stay thin; the page lives in `features::auth::view`.
#[component]
fn AuthRoute() -> impl IntoElement {
    crate::features::auth::view::auth_page::AuthPage::new()
}
