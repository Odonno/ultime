use leptos::*;

#[server(PublishPost, "/api")]
pub async fn publish_post(cx: Scope, post_id: String) -> Result<(), ServerFnError> {
    use serde::{Deserialize, Serialize};
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

    use crate::db::events::publish_post::PublishPostData;

    let request = expect_context::<actix_web::HttpRequest>(cx);

    let token = request
        .cookie("access_token")
        .and_then(|cookie| {
            let binding = cookie.clone();
            let value = binding.value();
            Some(value.to_string())
        })
        .ok_or(ServerFnError::ServerError("Cannot get token".to_string()))?;

    let db = Surreal::new::<Ws>("localhost:8000").await.map_err(|_| {
        ServerFnError::ServerError("Cannot open connection to SurrealDB".to_string())
    })?;

    db.use_ns("test")
        .use_db("test")
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot use namespace and database".to_string()))?;

    db.authenticate(token)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot authenticate".to_string()))?;

    let data = PublishPostData {
        post_id: ("post".to_string(), post_id.to_string()).into(),
    };

    crate::db::events::publish_post::publish_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot publish post".to_string()))?;

    Ok(())
}
