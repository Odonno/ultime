use components::App;

mod components;

fn main() {
    leptos::mount_to_body(|cx| view! { cx, <App /> })
}
