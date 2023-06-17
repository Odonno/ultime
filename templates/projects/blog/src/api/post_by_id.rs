use leptos::*;

use crate::models::queries::PostQuery;

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

    const QUERY: &str = include_str!("../../queries/post.surql");

    let post: PostQuery = db
        .query(QUERY)
        .bind(("post_id", post_id))
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot get post details".to_string()))?
        .take(0)
        .map_err(|_| ServerFnError::ServerError("Cannot get post details".to_string()))?;

    Ok(post)
}
