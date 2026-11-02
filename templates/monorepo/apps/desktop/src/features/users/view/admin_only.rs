//! `AdminOnly`: the users screens' gate, as `apps/api`'s users routes are admin-only.

use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_state::AuthStatus;
use rok_ui::prelude::*;

/// Shows `children` to an admin. Anyone else is told the screen needs the admin role, and
/// nothing asks the API for it; while the signed-in account is on its way, a spinner.
#[component]
pub fn AdminOnly(#[children] children: Vec<AnyElement>, cx: &mut Cx) -> impl IntoElement {
    let state = cx.watch_bloc::<AuthBloc>();
    match (&state.user, state.status) {
        (Some(user), _) if user.is_admin() => div().size_full().children(children),
        (Some(_), _) => div()
            .debug_selector(|| "admin-required".into())
            .p_6()
            .child(
                Empty::new()
                    .icon(IconName::User)
                    .title("This needs the admin role")
                    .description(
                        "Only an admin sees every user. Your own account is under My account.",
                    ),
            ),
        (None, AuthStatus::Failure) => div().p_6().child(
            Alert::new("Could not load your account")
                .destructive()
                .description(
                    state
                        .error
                        .as_ref()
                        .map_or_else(|| "Something went wrong.".to_owned(), ToString::to_string),
                ),
        ),
        (None, _) => div().p_6().child(Spinner::new()),
    }
}
