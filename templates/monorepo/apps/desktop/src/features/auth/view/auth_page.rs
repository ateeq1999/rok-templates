//! The `auth` page: sign in, create an account, finish a two-factor sign-in, sign out.

use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_event::AuthEvent;
use crate::features::auth::bloc::auth_state::{AuthState, AuthStatus};
use rok_ui::bloc::BlocListener;
use rok_ui::form::ServerErrorOptions;
use rok_ui::form::{self, FormOptions, FormValues, SubmitButton};
use rok_ui::prelude::*;

/// What the sign-in and sign-up form edits: `apps/api` names its fields the same.
#[derive(FormValues, Clone, Default)]
struct CredentialsForm {
    /// `email`.
    email: String,
    /// `password`.
    password: String,
}

/// What the two-factor form edits.
#[derive(FormValues, Clone, Default)]
struct CodeForm {
    /// An authenticator code, or a recovery code.
    code: String,
}

/// Where API errors from the credentials form are shown. A 422's broken rules land on the
/// fields the API names (`email`, `password`); a 401 (wrong credentials) under the password.
/// Other answers, such as the 403 of an unverified email, show as the page's message.
fn server_errors() -> ServerErrorOptions {
    ServerErrorOptions::new()
        .bad_request_field(&CredentialsForm::EMAIL)
        .unauthorized_field(&CredentialsForm::PASSWORD)
}

/// What the page says under the forms: progress, `apps/api`'s answer, or what to do next.
fn message(state: &AuthState) -> Option<String> {
    match state.status {
        AuthStatus::Loading => Some("Loading...".to_owned()),
        AuthStatus::Failure => Some(
            state
                .error
                .as_ref()
                .map_or_else(|| "Something went wrong.".to_owned(), ToString::to_string),
        ),
        _ => state.signed_up.as_ref().map(|email| {
            format!("Account created. Open the link mailed to {email} to verify it, then sign in.")
        }),
    }
}

/// Shows `auth` and adds its events.
#[component]
pub fn AuthPage(cx: &mut Cx) -> impl IntoElement {
    let bloc = cx.bloc::<AuthBloc>();
    let state = cx.watch_bloc::<AuthBloc>();
    // Whether the credentials create an account rather than sign in.
    let creating = cx.use_state(|| false);
    let submitting = bloc.clone();
    let creating_now = creating.clone();
    let credentials = form::use_form(
        cx,
        FormOptions::new(CredentialsForm::default()).on_submit(
            move |values: CredentialsForm, cx| {
                let (email, password) = (values.email, values.password);
                submitting.add(if creating_now.get(cx) {
                    AuthEvent::SignUpSubmitted { email, password }
                } else {
                    AuthEvent::SignInSubmitted { email, password }
                });
                gpui::Task::ready(Ok(()))
            },
        ),
    );
    let verifying = bloc.clone();
    let code = form::use_form(
        cx,
        FormOptions::new(CodeForm::default()).on_submit(move |values: CodeForm, _| {
            verifying.add(AuthEvent::CodeSubmitted { code: values.code });
            gpui::Task::ready(Ok(()))
        }),
    );
    // The bloc's error, on the credentials form's fields.
    let errors_form = credentials.clone();
    let errors = BlocListener::new(&bloc, move |state, _, cx| {
        if let Some(error) = &state.error {
            errors_form.apply_server_errors(cx, &form::to_server_errors(error, &server_errors()));
        }
    })
    .listen_when(|previous, current| previous.error != current.error);

    let body = if let Some(user) = &state.user {
        let handler = bloc.clone();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(format!("Signed in as {}.", user.email))
            .child(
                Button::new("sign-out")
                    .outline()
                    .label("Sign out")
                    .on_click(move |_, _, _| handler.add(AuthEvent::SignOutRequested)),
            )
    } else if state.code_required() {
        let handler = bloc.clone();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child("Enter the code from your authenticator app, or a recovery code.")
            .child(form::TextField::new(
                &code.field(cx, CodeForm::CODE),
                "Code",
            ))
            .child(SubmitButton::new(&code, "Verify"))
            .child(
                Button::new("code-cancelled")
                    .ghost()
                    .label("Back")
                    .on_click(move |_, _, _| handler.add(AuthEvent::CodeCancelled)),
            )
    } else {
        let is_creating = creating.get(cx);
        let toggle = creating.clone();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(form::TextField::new(
                &credentials.field(cx, CredentialsForm::EMAIL),
                "Email",
            ))
            .child(
                form::TextField::new(
                    &credentials.field(cx, CredentialsForm::PASSWORD),
                    "Password",
                )
                .password(),
            )
            .child(SubmitButton::new(
                &credentials,
                if is_creating {
                    "Create account"
                } else {
                    "Sign in"
                },
            ))
            .child(
                Button::new("auth-mode")
                    .link()
                    .label(if is_creating {
                        "I have an account: sign in"
                    } else {
                        "New here? Create an account"
                    })
                    .on_click(move |_, _, cx| toggle.update(cx, |creating| *creating = !*creating)),
            )
    };

    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(H2::new(if state.user.is_some() {
            "Account"
        } else {
            "Sign in"
        }))
        .child(body)
        .child(errors)
        .children(message(&state).map(|text| div().child(text)))
}
