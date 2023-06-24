use chrono::DateTime;
use leptos::{ev::SubmitEvent, *};
use leptos_router::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    api::{fetch_post_details, publish_post, unpublish_post, CommentPostOrComment, CommentTarget},
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

    let created_at = DateTime::parse_from_rfc3339(&post.created_at).unwrap_or(DateTime::default());
    let created_at = created_at.format("%B %e, %Y %H:%M").to_string();

    view! {
        cx,
        <header class="post-details-header">
            <h1>{post.title}</h1>

            <div class="post-details-header-infos">
                <span>{post.status}</span>
                <span>" | "</span>
                <span>{created_at} " by " {post.author}</span>
            </div>

            <p inner_html={post.content} />

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
        </header>

        <section class="comment-section">
            <CommentForm target={CommentTarget::BlogPost(post.id)} fetch_post_details={fetch_post_details} />

            <Comments comments={post.comments} fetch_post_details={fetch_post_details} />
        </section>
    }
}

#[component]
pub fn Comments(
    cx: Scope,
    comments: Vec<PostByIdQueryComments>,
    fetch_post_details: Resource<
        Result<String, PostDetailsPageError>,
        Result<PostByIdQueryItem, PostDetailsPageError>,
    >,
) -> impl IntoView {
    view! {
        cx,
        <ul class="comment-list">
            <For
                each={move || comments.clone()}
                key=|comment| comment.id.to_string()
                view=move |cx, comment| {
                    let created_at = DateTime::parse_from_rfc3339(&comment.created_at)
                        .unwrap_or(DateTime::default());
                    let created_at = created_at.format("%B %e, %Y %H:%M").to_string();

                    view! {
                        cx,
                        <section class="comment-section">
                            <div class="comment-infos">
                                <span>{created_at}</span>
                                <span>" by "</span>
                                <span>{comment.author}</span>
                            </div>

                            <p>{comment.content}</p>

                            <CommentForm target={CommentTarget::Comment(comment.id)} fetch_post_details={fetch_post_details} />

                            <Comments comments={comment.comments} fetch_post_details={fetch_post_details} />
                        </section>
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn CommentForm(
    cx: Scope,
    target: CommentTarget,
    fetch_post_details: Resource<
        Result<String, PostDetailsPageError>,
        Result<PostByIdQueryItem, PostDetailsPageError>,
    >,
) -> impl IntoView {
    let submit_comment = create_server_action::<CommentPostOrComment>(cx);

    let on_submit = move |ev: SubmitEvent| {
        let data = CommentPostOrComment::from_event(&ev);

        if data.is_err() {
            ev.prevent_default();
        }

        let data = data.unwrap();

        if data.content.is_empty() {
            ev.prevent_default();
        }
    };

    let pending = submit_comment.pending();

    create_effect(cx, move |_| match submit_comment.value().get() {
        Some(Ok(_)) => {
            fetch_post_details.refetch();
        }
        _ => {}
    });

    // TODO : local storage for textarea content

    view! {
        cx,
        <ActionForm action=submit_comment on:submit=on_submit>
            <input type="hidden" name="target" value=target />
            <textarea name="content" placeholder="Your comment" />
            <button type="submit" class="submit-comment-button" disabled=pending>
                "Submit comment"
            </button>
        </ActionForm>
    }
}
