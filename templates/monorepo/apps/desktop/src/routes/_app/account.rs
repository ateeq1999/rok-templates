use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    component: AccountRoute,
}

/// `/account`: the signed-in user's account. Routes stay thin; the page lives in
/// `features::auth::view`.
#[component]
fn AccountRoute() -> impl IntoElement {
    crate::features::auth::view::account_page::AccountPage::new()
}
