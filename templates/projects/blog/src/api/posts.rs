use leptos::*;

use crate::models::queries::PostsQuery;

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

    const QUERY: &str = include_str!("../../queries/posts.surql");

    let posts: PostsQuery = db
        .query(QUERY)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot get all posts".to_string()))?
        .take(0)
        .map_err(|_| ServerFnError::ServerError("Cannot get all posts".to_string()))?;

    Ok(posts)
}
