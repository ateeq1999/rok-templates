//! `RestoredSession`: who a token kept from the last launch signs in as.

use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_event::AuthEvent;
use rok_ui::http::Session;
use rok_ui::prelude::*;

/// Once, on its first render, asks for the signed-in user's profile when the session started
/// with a token the keychain kept: the user is shown signed in, or a 401 (the token expired
/// meanwhile) sends them to sign in. Render it once, below the `AuthBloc`'s provider.
#[component]
pub fn RestoredSession(cx: &mut Cx) -> impl IntoElement {
    let bloc = cx.bloc::<AuthBloc>();
    let session = cx.repository::<Session>();
    cx.use_state(move || {
        if session.is_signed_in() {
            bloc.add(AuthEvent::ProfileRequested);
        }
    });
    div()
}
