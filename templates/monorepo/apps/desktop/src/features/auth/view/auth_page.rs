//! The `auth` page.

use crate::features::auth::bloc::auth_bloc::AuthBloc;
use crate::features::auth::bloc::auth_event::AuthEvent;
use crate::features::auth::bloc::auth_state::AuthStatus;
use rok_ui::bloc::BlocBuilder;
use rok_ui::bloc::BlocListener;
use rok_ui::form::ServerErrorOptions;
use rok_ui::form::{self, FormOptions, FormValues, SubmitButton};
use rok_ui::prelude::*;

/// What the `SignInSubmitted` form edits.
#[derive(FormValues, Clone, Default)]
struct SignInSubmittedForm {
    /// `email`.
    email: String,
    /// `password`.
    password: String,
}

/// Where API errors from submitting the form are shown.
fn server_errors() -> ServerErrorOptions {
    ServerErrorOptions::new()
        .map("emailAddress", &SignInSubmittedForm::EMAIL)
        .bad_request_field(&SignInSubmittedForm::EMAIL)
        .unauthorized_field(&SignInSubmittedForm::PASSWORD)
}

/// Shows `auth` and adds its events.
#[component]
pub fn AuthPage(cx: &mut Cx) -> impl IntoElement {
    let bloc = cx.bloc::<AuthBloc>();
    let submitting = bloc.clone();
    let form = form::use_form(
        cx,
        FormOptions::new(SignInSubmittedForm::default()).on_submit(
            move |values: SignInSubmittedForm, _| {
                submitting.add(AuthEvent::SignInSubmitted {
                    email: values.email,
                    password: values.password,
                });
                gpui::Task::ready(Ok(()))
            },
        ),
    );
    let editor = div()
        .flex()
        .flex_col()
        .gap_2()
        .child(form::TextField::new(
            &form.field(cx, SignInSubmittedForm::EMAIL),
            "Email",
        ))
        .child(
            form::TextField::new(&form.field(cx, SignInSubmittedForm::PASSWORD), "Password")
                .password(),
        )
        .child(SubmitButton::new(&form, "Sign in"));
    // The bloc's error, on the form's fields.
    let errors_form = form.clone();
    let errors = BlocListener::new(&bloc, move |state, _, cx| {
        if let Some(error) = &state.error {
            errors_form.apply_server_errors(cx, &form::to_server_errors(error, &server_errors()));
        }
    })
    .listen_when(|previous, current| previous.error != current.error);
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_6()
        .child(H2::new("Auth"))
        .child(
            div()
                .flex()
                .gap_2()
                .child({
                    let handler = bloc.clone();
                    Button::new("sign-out-requested")
                        .outline()
                        .label("Request")
                        .on_click(move |_, _, _| {
                            handler.add(AuthEvent::SignOutRequested);
                        })
                })
                .child({
                    let handler = bloc.clone();
                    Button::new("profile-requested")
                        .outline()
                        .label("Request")
                        .on_click(move |_, _, _| {
                            handler.add(AuthEvent::ProfileRequested);
                        })
                }),
        )
        .child(editor)
        .child(errors)
        .child(BlocBuilder::new(&bloc, |state, _, _| match state.status {
            AuthStatus::Loading => div().child("Loading..."),
            AuthStatus::Failure => div().child(
                state
                    .error
                    .as_ref()
                    .map_or_else(|| "Something went wrong.".to_string(), ToString::to_string),
            ),
            _ => div(),
        }))
}
