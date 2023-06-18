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
        <h1>"Welcome to my blog!"</h1>

        <ActionForm action=create_blog_post on:submit=on_submit>
            <input
                type="text"
                name="title"
                placeholder="Title"
            />
            <input
                type="text"
                name="content"
                placeholder="Content"
            />

            <button type="submit" disabled=pending>"Create blog post."</button>
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
