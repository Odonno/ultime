use leptos::*;

#[server(PublishPost, "/api")]
pub async fn publish_post(post_id: String) -> Result<(), ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root, sql::Thing};
    use serde::{Serialize, Deserialize};
    
    use crate::db::events::publish_post::PublishPostData;

    let db = Surreal::new::<Ws>("localhost:8000")
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot open connection to SurrealDB".to_string()))?;

    db
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot signin to SurrealDB".to_string()))?;

    db
        .use_ns("test")
        .use_db("test")
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot use namespace and database".to_string()))?;

    let data = PublishPostData {
        post_id: ("post".to_string(), post_id.to_string()).into()
    };

    crate::db::events::publish_post::publish_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot publish post".to_string()))?;

    Ok(())
}