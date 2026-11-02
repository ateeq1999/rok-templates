//! A user's detail screen.

use crate::data::models::user::User;
use crate::features::users::bloc::user_bloc::UserBloc;
use crate::features::users::bloc::user_event::UserEvent;
use crate::features::users::bloc::user_state::{UserState, UserStatus};
use crate::shared::widgets::PageFrame;
use rok_ui::bloc::BlocBuilder;
use rok_ui::prelude::*;

/// Shows the user `id`, asking the bloc for them whenever the screen opens on another id.
#[component]
pub fn UserPage(id: SharedString, cx: &mut Cx) -> impl IntoElement {
    let bloc = cx.bloc::<UserBloc>();
    let shown = cx.use_state(|| None::<SharedString>);
    if shown.get(cx).as_ref() != Some(&id) {
        shown.set(Some(id.clone()), cx);
        bloc.add(UserEvent::Requested { id: id.to_string() });
    }
    PageFrame::new("User")
        .child(div().child(Link::new("back-to-users", "/users").child(Muted::new("← All users"))))
        .child(BlocBuilder::new(&bloc, |state, _, _| body(state)))
}

fn body(state: &UserState) -> AnyElement {
    match (&state.status, &state.user) {
        (UserStatus::Success, Some(user)) => detail(user).into_any_element(),
        (UserStatus::Failure, _) => Alert::new("Could not load this user")
            .destructive()
            .description(
                state
                    .error
                    .as_ref()
                    .map_or_else(|| "Something went wrong.".to_owned(), ToString::to_string),
            )
            .into_any_element(),
        _ => Spinner::new().into_any_element(),
    }
}

fn detail(user: &User) -> impl IntoElement {
    let name = if user.name.is_empty() {
        "No display name".to_owned()
    } else {
        user.name.clone()
    };
    div().debug_selector(|| "user-detail".into()).child(
        Card::new()
            .child(
                CardHeader::new()
                    .child(CardTitle::new(name))
                    .child(CardDescription::new(user.email.clone())),
            )
            .child(
                CardContent::new()
                    .child(field("Email", user.email.clone()))
                    .child(field("Id", user.id.clone())),
            ),
    )
}

/// A label over its value.
fn field(label: &'static str, value: String) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .py_2()
        .child(Muted::new(label))
        .child(value)
}
