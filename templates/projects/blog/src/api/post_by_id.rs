use leptos::*;

use crate::models::queries::PostByIdQueryItem;

#[server(FetchPostDetails, "/api")]
pub async fn fetch_post_details(
    post_id: String,
) -> Result<Option<PostByIdQueryItem>, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

    use crate::db::queries::post_by_id::query_post_by_id;

    let db = Surreal::new::<Ws>("localhost:8000").await.map_err(|_| {
        ServerFnError::ServerError("Cannot open connection to SurrealDB".to_string())
    })?;

    db.use_ns("test")
        .use_db("test")
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot use namespace and database".to_string()))?;

    let post = query_post_by_id(&db, post_id)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot get post details".to_string()))?;

    let post = post.first().cloned();

    Ok(post)
}
