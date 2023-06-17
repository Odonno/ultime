use leptos::*;
use leptos_router::*;

use crate::{api::fetch_blog_posts, models::queries::PostsQueryItem};

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let fetch_blog_posts_resource =
        create_resource(cx, || (), |_| async move { fetch_blog_posts().await });

    let (title, set_title) = create_signal(cx, "".to_string());
    let (content, set_content) = create_signal(cx, "".to_string());

    view! {
        cx,
        <h1>"Welcome to Leptos!"</h1>

        <form>
            <input
                type="text"
                name="title"
                placeholder="Title"
                on:input=move |ev| {
                    set_title(event_target_value(&ev));
                }
                prop:value={title}
            />
            <input
                type="text"
                name="content"
                placeholder="Content"
                on:input=move |ev| {
                    set_content(event_target_value(&ev));
                }
                prop:value={content}
            />

            <button>"Create blog post."</button>
        </form>

        <Transition fallback=move || { view! { cx, <div>"Loading..."</div> } }>
            <BlogPosts fetch_blog_posts_resource={fetch_blog_posts_resource} />
        </Transition>
    }
}

#[component]
pub fn BlogPosts(
    cx: Scope,
    fetch_blog_posts_resource: Resource<(), Result<Vec<PostsQueryItem>, ServerFnError>>,
) -> impl IntoView {
    let posts = move || {
        fetch_blog_posts_resource
            .read(cx)
            .map(|posts| posts.unwrap_or_default())
            .unwrap_or_default()
    };

    view! {
        cx,
        <ul>
            <For
                each={posts}
                key=|post| post.id.to_string()
                view=move |cx, post| {
                    view! {
                        cx,
                        <BlogPost post={post} />
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn BlogPost(cx: Scope, post: PostsQueryItem) -> impl IntoView {
    let href = format!("posts/{}", post.id);

    view! {
        cx,
        <li>
            <A href={href}>
                <h3>{post.title}</h3>
                <p>{post.content}</p>

                <div>{post.status}</div>
                <div>"Comments: " {post.number_of_comments}</div>
            </A>
        </li>
    }
}
