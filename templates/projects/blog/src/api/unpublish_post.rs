use leptos::*;

#[server(UnpublishPost, "/api")]
pub async fn unpublish_post(post_id: String) -> Result<(), ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root, sql::Thing};
    use serde::{Serialize, Deserialize};
    
    use crate::db::events::unpublish_post::UnpublishPostData;

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

    let data = UnpublishPostData {
        post_id: ("post".to_string(), post_id.to_string()).into()
    };

    crate::db::events::unpublish_post::unpublish_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot unpublish post".to_string()))?;

    Ok(())
}