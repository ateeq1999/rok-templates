//! The app's composition root: repositories are built once here, and blocs are provided
//! above the routes, so every page reads the same instances.
//!
//! `cargo rok-ui generate` adds lines at the `// rok-ui:` markers; keep them.

// Used by the lines `cargo rok-ui generate` adds.
#[allow(unused_imports)]
use std::sync::Arc;

use rok_ui::bloc::{BlocProvider, Repositories, RepositoryProvider};
use rok_ui::http::Session;
use rok_ui::prelude::*;

use crate::data::providers::api::Api;
use crate::data::repositories::auth_repository::{AuthRepository, AuthRepositoryImpl};
use crate::data::repositories::users_repository::{UsersRepository, UsersRepositoryImpl};
use crate::routes;

/// The repositories, built once for the app's lifetime. They share `api` (and its session).
#[allow(clippy::needless_pass_by_value)]
fn repositories(api: Api) -> Repositories {
    Repositories::new()
        .with::<dyn UsersRepository>(Arc::new(UsersRepositoryImpl::new(api.clone())))
        .with::<dyn AuthRepository>(Arc::new(AuthRepositoryImpl::new(api.clone())))
        // rok-ui:repositories
        // Views watch the session (see `shared::widgets::SessionGuard`).
        .with::<Session>(Arc::new(api.session().clone()))
}

/// Repositories, then blocs, then the routes, all signed in as `api`'s session.
#[component]
pub fn App(api: Api, cx: &mut Cx) -> impl IntoElement {
    let repositories = cx.use_state(move || repositories(api)).get(cx);
    RepositoryProvider::from(repositories).child(
        BlocProvider::new()
            .with_bloc(|scope| {
                crate::features::auth::bloc::auth_bloc::AuthBloc::new(
                    scope.repository::<dyn AuthRepository>(),
                )
            })
            .with_bloc(|scope| {
                crate::features::users::bloc::users_bloc::UsersBloc::new(
                    scope.repository::<dyn UsersRepository>(),
                )
            })
            .with_bloc(|scope| {
                crate::features::users::bloc::user_bloc::UserBloc::new(
                    scope.repository::<dyn UsersRepository>(),
                )
            })
            // rok-ui:blocs
            // A token the keychain kept signs the user in at start.
            .child(crate::features::auth::view::restored_session::RestoredSession::new())
            .child(routes::tree()),
    )
}
