use rok_ui::prelude::*;
use rok_ui::router::file_route;

file_route! {
    layout: UsersLayout,
}

/// Around `/users` and `/users/:id`: they are `apps/api`'s admin routes, so only an admin
/// sees them; anyone else is told they need the admin role.
#[component]
fn UsersLayout(#[children] children: Vec<AnyElement>) -> impl IntoElement {
    crate::features::users::view::admin_only::AdminOnly::new().children(children)
}
