//! "My account": the signed-in user's own account, which every user reaches.

use crate::data::models::user::User;
use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_state::{AuthState, AuthStatus};
use crate::shared::widgets::{field, PageFrame};
use rok_ui::prelude::*;

/// Shows the signed-in user's account, as `apps/api` answered it at sign-in (or at start, for
/// a token the keychain kept).
#[component]
pub fn AccountPage(cx: &mut Cx) -> impl IntoElement {
    let state = cx.watch_bloc::<AuthBloc>();
    PageFrame::new("My account").child(body(&state))
}

fn body(state: &AuthState) -> AnyElement {
    match (&state.user, state.status) {
        (Some(user), _) => detail(user).into_any_element(),
        (None, AuthStatus::Failure) => Alert::new("Could not load your account")
            .destructive()
            .description(
                state
                    .error
                    .as_ref()
                    .map_or_else(|| "Something went wrong.".to_owned(), ToString::to_string),
            )
            .into_any_element(),
        // A kept token's account is on its way.
        (None, _) => Spinner::new().into_any_element(),
    }
}

fn detail(user: &User) -> impl IntoElement {
    let name = if user.name.is_empty() {
        "No display name".to_owned()
    } else {
        user.name.clone()
    };
    let verified = if user.email_verified {
        "Verified"
    } else {
        "Not verified yet"
    };
    div().debug_selector(|| "account".into()).child(
        Card::new()
            .child(
                CardHeader::new()
                    .child(CardTitle::new(name))
                    .child(CardDescription::new(user.email.clone())),
            )
            .child(
                CardContent::new()
                    .child(field("Email", user.email.clone()))
                    .child(field("Email address", verified))
                    .child(field("Role", user.role.as_str()))
                    .child(field("Id", user.id.clone())),
            ),
    )
}
