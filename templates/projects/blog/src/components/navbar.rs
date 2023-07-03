use leptos::*;
use leptos_router::*;

use crate::{api::fetch_navbar, models::queries::NavbarQueryItem};

#[component]
pub fn Navbar(cx: Scope) -> impl IntoView {
    let fetch_navbar_resource = create_resource(
        cx,
        move || cx.clone(),
        |cx| async move { fetch_navbar(cx).await },
    );

    view! {
        cx,
        <nav class="main-navbar">
            <A href="/">"Home"</A>

            <Transition fallback=move || { view! { cx, <div /> } }>
                <RightNavbar fetch_navbar_resource={fetch_navbar_resource} />
            </Transition>
        </nav>
    }
}

#[component]
pub fn RightNavbar(
    cx: Scope,
    fetch_navbar_resource: Resource<Scope, Result<NavbarQueryItem, ServerFnError>>,
) -> impl IntoView {
    let navbar = move || {
        fetch_navbar_resource
            .read(cx)
            .map(|navbar| navbar.ok())
            .unwrap_or(None)
    };

    view! {
        cx,
        {move || match navbar() {
            None => {
                view! { cx, <AnonymousNavbar /> }.into_view(cx)
            },
            Some(value) => {
                view! { cx, <AuthenticatedNavbar username={value.username} avatar={value.avatar} /> }.into_view(cx)
            }
        }}
    }
}

#[component]
pub fn AnonymousNavbar(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <ul class="anonymous-navbar-list">
            <li>
                <A href="/sign_up" class="sign-up-link">"Sign Up"</A>
            </li>
            <li>
                <A href="/sign_in" class="sign-in-link">"Sign In"</A>
            </li>
        </ul>
    }
}

#[component]
pub fn AuthenticatedNavbar(cx: Scope, username: String, avatar: String) -> impl IntoView {
    view! {
        cx,
        <ul class="authenticated-navbar-list">
            <li class="username">{username}</li>
            <li>
                <img src={&avatar} alt={avatar + "?s=32"} class="avatar" />
            </li>
            <li>
                <A href="/sign_out" class="sign-out-link">"Sign Out"</A>
            </li>
        </ul>
    }
}
