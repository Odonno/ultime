use leptos::*;
use leptos_router::*;

use crate::{
    api::fetch_post_details,
    models::queries::{PostByIdQueryComments, PostByIdQueryItem},
};

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
pub fn BlogPost(cx: Scope, post: PostByIdQueryItem) -> impl IntoView {
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
