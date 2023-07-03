use leptos::{ev::SubmitEvent, *};
use leptos_router::*;

use crate::api::SignUp;
use crate::components::navbar::Navbar;

#[component]
pub fn SignUpPage(cx: Scope) -> impl IntoView {
    let sign_up = create_server_action::<SignUp>(cx);

    let on_submit = move |ev: SubmitEvent| {
        let data = SignUp::from_event(&ev);

        if data.is_err() {
            ev.prevent_default();
        }

        let data = data.unwrap();

        if data.username.is_empty()
            || data.email.is_empty()
            || data.confirm_email.is_empty()
            || data.password.is_empty()
            || data.confirm_password.is_empty()
            || data.email != data.confirm_email
            || data.password != data.confirm_password
            || data.password.len() < 8
        {
            ev.prevent_default();
        }
    };

    let pending = sign_up.pending();

    view! {
        cx,
        <Navbar />

        <ActionForm action=sign_up on:submit=on_submit class="sign-up-form">
            <h1>"Sign up"</h1>

            <input
                type="text"
                name="username"
                placeholder="Username"
            />

            <input
                class="email-input"
                type="email"
                name="email"
                placeholder="Email"

            />
            <input
                type="email"
                name="confirm_email"
                placeholder="Confirm email"

            />

            <input
                class="password-input"
                type="password"
                name="password"
                placeholder="Password"
            />
            <input
                type="password"
                name="confirm_password"
                placeholder="Confirm password"
            />

            <div>
                <button type="submit" disabled=pending>"Sign up"</button>
            </div>
        </ActionForm>
    }
}
