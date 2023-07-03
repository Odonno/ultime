use leptos::{ev::SubmitEvent, *};
use leptos_router::*;

use crate::api::SignIn;
use crate::components::navbar::Navbar;

#[component]
pub fn SignInPage(cx: Scope) -> impl IntoView {
    let sign_in = create_server_action::<SignIn>(cx);

    let on_submit = move |ev: SubmitEvent| {
        let data = SignIn::from_event(&ev);

        if data.is_err() {
            ev.prevent_default();
        }

        let data = data.unwrap();

        if data.username.is_empty() || data.password.is_empty() || data.password.len() < 8 {
            ev.prevent_default();
        }
    };

    let pending = sign_in.pending();

    view! {
        cx,
        <Navbar />

        <ActionForm action=sign_in on:submit=on_submit class="sign-in-form">
            <h1>"Sign in"</h1>

            <input
                type="text"
                name="username"
                placeholder="Username"
            />

            <input
                type="password"
                name="password"
                placeholder="Password"
            />

            <div>
                <button type="submit" disabled=pending>"Sign in"</button>
            </div>
        </ActionForm>
    }
}
