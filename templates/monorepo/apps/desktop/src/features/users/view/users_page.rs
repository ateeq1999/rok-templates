//! The users list: a table of the page the API answered, and its pagination.

use super::user_path;
use crate::data::models::user::User;
use crate::features::users::bloc::users_bloc::UsersBloc;
use crate::features::users::bloc::users_event::UsersEvent;
use crate::features::users::bloc::users_state::{UsersState, UsersStatus};
use crate::shared::widgets::PageFrame;
use rok_ui::bloc::{BlocBuilder, BlocHandle};
use rok_ui::prelude::*;

/// Shows the users, a page at a time. Opening it (re)loads the page it last showed.
#[component]
pub fn UsersPage(cx: &mut Cx) -> impl IntoElement {
    let bloc = cx.bloc::<UsersBloc>();
    let opened = cx.use_state(|| false);
    if !opened.get(cx) {
        opened.set(true, cx);
        let page = bloc.state().page.page.max(1);
        bloc.add(UsersEvent::PageRequested { page });
    }
    let paging = bloc.clone();
    PageFrame::new("Users").child(BlocBuilder::new(&bloc, move |state, _, _| {
        body(state, &paging)
    }))
}

fn body(state: &UsersState, bloc: &BlocHandle<UsersBloc>) -> AnyElement {
    match state.status {
        UsersStatus::Failure => Alert::new("Could not load the users")
            .destructive()
            .description(
                state
                    .error
                    .as_ref()
                    .map_or_else(|| "Something went wrong.".to_owned(), ToString::to_string),
            )
            .into_any_element(),
        UsersStatus::Success if state.page.items.is_empty() => Empty::new()
            .icon(IconName::User)
            .title("No users yet")
            .into_any_element(),
        // The first page is on its way: nothing to keep showing yet.
        _ if state.page.items.is_empty() => Spinner::new().into_any_element(),
        UsersStatus::Initial | UsersStatus::Loading | UsersStatus::Success => div()
            .flex()
            .flex_col()
            .gap_4()
            .child(table(&state.page.items))
            .child(pagination(state, bloc))
            .into_any_element(),
    }
}

fn table(users: &[User]) -> impl IntoElement {
    div().debug_selector(|| "users-table".into()).child(
        Table::new()
            .child(
                TableHeader::new().child(
                    TableRow::new()
                        .child(TableHead::new("Name"))
                        .child(TableHead::new("Email")),
                ),
            )
            .child(TableBody::new().children(users.iter().map(row))),
    )
}

/// A user's row; clicking it opens their detail.
fn row(user: &User) -> impl IntoElement {
    let name = if user.name.is_empty() {
        "—".to_owned()
    } else {
        user.name.clone()
    };
    let path = user_path(&user.id);
    div()
        .id(SharedString::from(format!("user-row-{}", user.id)))
        .debug_selector({
            let selector = format!("user-row-{}", user.id);
            move || selector
        })
        .cursor_pointer()
        .on_click(move |_, _, cx| rok_ui::router::navigate(path.clone(), cx))
        .child(
            TableRow::new()
                .child(TableCell::new().child(name))
                .child(TableCell::new().child(user.email.clone())),
        )
}

/// The pages, as the API counts them; picking one asks the bloc for it.
fn pagination(state: &UsersState, bloc: &BlocHandle<UsersBloc>) -> impl IntoElement {
    let bloc = bloc.clone();
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(Muted::new(format!(
            "{} users, page {} of {}",
            state.page.total,
            state.page.page,
            state.page_count()
        )))
        .child(
            Pagination::new("users-pages", state.page_count() as usize)
                .current_page(state.page.page as usize)
                .on_change(move |page, _, _| {
                    let page = u32::try_from(*page).unwrap_or(1);
                    bloc.add(UsersEvent::PageRequested { page });
                }),
        )
}
