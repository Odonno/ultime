use leptos::{ev::SubmitEvent, *};
use leptos_router::*;

use crate::{
    api::{fetch_blog_posts, CreateBlogPost},
    models::queries::PostsQueryItem,
};

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let fetch_blog_posts_resource =
        create_resource(cx, || (), |_| async move { fetch_blog_posts().await });

    let create_blog_post = create_server_action::<CreateBlogPost>(cx);

    let on_submit = move |ev: SubmitEvent| {
        let data = CreateBlogPost::from_event(&ev);

        if data.is_err() {
            ev.prevent_default();
        }

        let data = data.unwrap();

        if data.title.is_empty() || data.content.is_empty() {
            ev.prevent_default();
        }
    };

    let pending = create_blog_post.pending();

    view! {
        cx,
        <header class="post-list-header">
            <h1>"Welcome to my blog!"</h1>
        </header>

        <ActionForm action=create_blog_post on:submit=on_submit class="new-post-form">
            <input
                type="text"
                name="title"
                placeholder="Title"
            />
            <textarea
                type="text"
                name="content"
                placeholder="Content"
                rows=4
            />

            <div>
                <button type="submit" disabled=pending>"Create blog post"</button>
            </div>
        </ActionForm>

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
        <ul class="post-list">
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
                <h2>{post.title}</h2>
                <p inner_html={post.content} />

                <div class="post-list-footer">
                    <span>{post.status}</span>
                    <span>" | "</span>
                    <span>{post.number_of_comments} " comments"</span>
                </div>
            </A>
        </li>
    }
}
