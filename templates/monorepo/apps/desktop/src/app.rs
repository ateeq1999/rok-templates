//! The app's composition root: repositories are built once here, and blocs are provided
//! above the routes, so every page reads the same instances.
//!
//! `cargo rok-ui generate` adds lines at the `// rok-ui:` markers; keep them.

// Used by the lines `cargo rok-ui generate` adds.
#[allow(unused_imports)]
use std::sync::Arc;

use rok_ui::bloc::{BlocProvider, Repositories, RepositoryProvider};
use rok_ui::http::{HttpClient, Session};
use rok_ui::prelude::*;

use crate::routes;

/// The repositories, built once for the app's lifetime. HTTP providers share `client`.
#[allow(clippy::needless_pass_by_value)]
fn repositories(client: HttpClient, session: Session) -> Repositories {
    Repositories::new()
        // Views watch the session (see `shared::widgets::SessionGuard`).
        .with::<Session>(Arc::new(session))
        .with::<dyn crate::data::repositories::notes_repository::NotesRepository>(Arc::new(crate::data::repositories::notes_repository::NotesRepositoryImpl::new(crate::data::providers::notes_api::NotesApi::new(client.clone()))))
        .with::<dyn crate::data::repositories::auth_repository::AuthRepository>(Arc::new(crate::data::repositories::auth_repository::AuthRepositoryImpl::new(crate::data::providers::auth_api::AuthApi::new(client.clone()))))
        // rok-ui:repositories
}

/// Repositories, then blocs, then the routes.
#[component]
pub fn App(client: HttpClient, session: Session, cx: &mut Cx) -> impl IntoElement {
    let repositories = cx.use_state(move || repositories(client, session)).get(cx);
    RepositoryProvider::from(repositories).child(
        BlocProvider::new()
            .with_bloc(|scope| crate::features::notes::bloc::notes_bloc::NotesBloc::new(scope.repository::<dyn crate::data::repositories::notes_repository::NotesRepository>()))
            .with_bloc(|scope| crate::features::auth::bloc::auth_bloc::AuthBloc::new(scope.repository::<dyn crate::data::repositories::auth_repository::AuthRepository>()))
            // rok-ui:blocs
            .child(routes::tree()),
    )
}
