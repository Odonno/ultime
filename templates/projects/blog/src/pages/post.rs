use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    api::{fetch_post_details, publish_post, unpublish_post},
    models::queries::{PostByIdQueryComments, PostByIdQueryItem},
};

#[derive(Params, PartialEq, Clone, Debug)]
struct PostDetailsPageParams {
    #[params]
    id: String,
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostDetailsPageError {
    #[error("Invalid post ID.")]
    InvalidId,
    #[error("Post not found.")]
    PostNotFound,
    #[error("Server error.")]
    ServerError,
}

#[component]
pub fn PostDetailsPage(cx: Scope) -> impl IntoView {
    let params = use_params::<PostDetailsPageParams>(cx);

    let id = move || {
        params.with(|p| {
            p.as_ref()
                .map(|p| p.id.to_string())
                .map_err(|_| PostDetailsPageError::InvalidId)
        })
    };

    let fetch_post_details = create_resource(cx, id, |id| async move {
        match id {
            Err(e) => Err(e),
            Ok(id) => fetch_post_details(id)
                .await
                .map(|data| data.ok_or(PostDetailsPageError::PostNotFound))
                .map_err(|_| PostDetailsPageError::ServerError)
                .flatten(),
        }
    });

    let post_view = move || {
        fetch_post_details.with(cx, |post_result| {
            post_result.clone().map(|post| {
                view! {
                    cx,
                    <BlogPost post=post fetch_post_details=fetch_post_details />
                }
            })
        })
    };

    view! {
        cx,
        <Suspense fallback=move || view! { cx, <p>"Loading..."</p> }>
            <ErrorBoundary fallback=|cx, errors| {
                view! { cx,
                    <div class="error">
                        <h1>"Something went wrong."</h1>
                        <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, error)| view! { cx, <li>{error.to_string()} </li> })
                            .collect_view(cx)
                        }
                        </ul>
                    </div>
                }
            }>
                {post_view}
            </ErrorBoundary>
        </Suspense>
    }
}

#[component]
pub fn BlogPost(
    cx: Scope,
    post: PostByIdQueryItem,
    fetch_post_details: Resource<
        Result<String, PostDetailsPageError>,
        Result<PostByIdQueryItem, PostDetailsPageError>,
    >,
) -> impl IntoView {
    let is_draft = post.status == "DRAFT";

    let post_id = post.id.to_string();

    view! {
        cx,
        <h1>{post.title}</h1>
        <div>{post.status}</div>
        <div>{post.created_at} " " {post.author}</div>
        <p>{post.content}</p>

        {move || {
            let post_id = post_id.to_string();

            if is_draft {
                let publish_closure = move |post_id| {
                    spawn_local(async move {
                        let _ = publish_post(post_id).await;
                        fetch_post_details.refetch();
                    });
                };

                view! {
                    cx,
                    <button type="button" on:click=move |_| publish_closure(post_id.to_string())>
                        "Publish post"
                    </button>
                }
            } else {
                let unpublish_closure = move |post_id| {
                    spawn_local(async move {
                        let _ = unpublish_post(post_id).await;
                        fetch_post_details.refetch();
                    });
                };

                view! {
                    cx,
                    <button type="button" on:click=move |_| unpublish_closure(post_id.to_string())>
                        "Unpublish post"
                    </button>
                }
            }
        }}

        <Comments comments={post.comments} />
    }
}

#[component]
pub fn Comments(cx: Scope, comments: Vec<PostByIdQueryComments>) -> impl IntoView {
    view! {
        cx,
        <ul>
            <For
                each={move || comments.clone()}
                key=|comment| comment.id.to_string()
                view=move |cx, comment| {
                    view! {
                        cx,
                        <div>{comment.created_at} " " {comment.author}</div>
                        <p>{comment.content}</p>

                        <Comments comments={comment.comments} />
                    }
                }
            />
        </ul>
    }
}
