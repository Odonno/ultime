use leptos::*;
use leptos_router::*;

use crate::api::SignOut;

#[component]
pub fn SignOutPage(cx: Scope) -> impl IntoView {
    let sign_out = create_server_action::<SignOut>(cx);

    let pending = sign_out.pending();

    create_effect(cx, move |_| {
        let value = SignOut {};
        sign_out.dispatch(value);
    });

    view! {
        cx,
        <Show
            when=move || sign_out.value().get().is_some()
            fallback=|_| view! { cx, "Signing out!" }
        >
            <Redirect
                path="/"
                options=NavigateOptions {
                    replace: true,
                    ..Default::default()
                }
            />
        </Show>

        <ActionForm action=sign_out class="sign-out-form">
            <button type="submit" disabled=pending></button>
        </ActionForm>
    }
}
