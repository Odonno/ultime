use leptos::*;
use leptos_router::*;

use crate::models::queries::{PostQuery, PostQueryComments, PostQueryItem};

#[server(FetchPostDetails, "/api")]
pub async fn fetch_post_details(post_id: String) -> Result<PostQuery, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

    let db = surrealdb::Surreal::new::<Ws>("localhost:8000")
        .await
        .map_err(|_| {
            ServerFnError::ServerError("Cannot open connection to SurrealDB".to_string())
        })?;

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await
    .map_err(|_| ServerFnError::ServerError("Cannot signin to SurrealDB".to_string()))?;

    db.use_ns("test")
        .use_db("test")
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot use namespace and database".to_string()))?;

    let query = include_str!("../../queries/post.surql");

    let post: PostQuery = db
        .query(query)
        .bind(("post_id", post_id))
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot get post details".to_string()))?
        .take(0)
        .map_err(|_| ServerFnError::ServerError("Cannot get post details".to_string()))?;

    Ok(post)
}

#[derive(Params, PartialEq, Clone, Debug)]
struct PostDetailsPageParams {
    #[params]
    id: String,
}

#[component]
pub fn PostDetailsPage(cx: Scope) -> impl IntoView {
    let params = use_params::<PostDetailsPageParams>(cx);

    let id = move || params().unwrap().id;

    let fetch_post_details =
        create_resource(cx, id, |id| async move { fetch_post_details(id).await });

    view! {
        cx,
        <Transition fallback=move || { view! { cx, <div>"Loading..."</div> } }>
            {move || match fetch_post_details.read(cx) {
                None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
                Some(query) => {
                    let post = query.unwrap().first().unwrap().clone();
                    view! { cx, <BlogPost post={post}/> }.into_view(cx)
                }
            }}
        </Transition>
    }
}

#[component]
pub fn BlogPost(cx: Scope, post: PostQueryItem) -> impl IntoView {
    view! {
        cx,
        <h1>{post.title}</h1>
        <div>{post.status}</div>
        <div>{post.created_at} " " {post.author}</div>
        <p>{post.content}</p>

        <Comments comments={post.comments} />
    }
}

#[component]
pub fn Comments(cx: Scope, comments: Vec<PostQueryComments>) -> impl IntoView {
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
