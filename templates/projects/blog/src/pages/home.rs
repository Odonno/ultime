use leptos::*;
use leptos_router::*;

use crate::models::queries::{PostsQuery, PostsQueryItem};

#[server(FetchBlogPosts, "/api")]
pub async fn fetch_blog_posts() -> Result<PostsQuery, ServerFnError> {
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

    let query = include_str!("../../queries/posts.surql");

    let posts: PostsQuery = db
        .query(query)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot get all posts".to_string()))?
        .take(0)
        .map_err(|_| ServerFnError::ServerError("Cannot get all posts".to_string()))?;

    Ok(posts)
}

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let fetch_blog_posts_resource =
        create_resource(cx, || (), |_| async move { fetch_blog_posts().await });

    let blog_posts = create_memo(cx, move |_| {
        fetch_blog_posts_resource
            .read(cx)
            .map(|result| result.unwrap())
            .unwrap_or_else(|| vec![])
    });

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
            <BlogPosts blog_posts={blog_posts} />
        </Transition>
    }
}

#[component]
pub fn BlogPosts(cx: Scope, blog_posts: Memo<Vec<PostsQueryItem>>) -> impl IntoView {
    view! {
        cx,
        <ul>
            <For
                each={blog_posts}
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
    let href = format!("posts/{}", post.id.id);

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
