use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    params: { id: String },
    component: UserRoute,
}

/// `/users/:id`: a user's detail. Routes stay thin; the page lives in `features::users::view`.
#[component]
fn UserRoute(cx: &mut Cx) -> impl IntoElement {
    let Route { id } = params(cx);
    crate::features::users::view::user_page::UserPage::new(id)
}
