use leptos::*;

use crate::models::queries::NavbarQueryItem;

#[server(FetchNavbar, "/api")]
pub async fn fetch_navbar(cx: Scope) -> Result<NavbarQueryItem, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

    use crate::db::queries::navbar::query_navbar;

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

    let result = query_navbar(&db)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot query navbar".to_string()))?;

    let result = result
        .first()
        .cloned()
        .ok_or(ServerFnError::ServerError("Cannot get navbar".to_string()))?;

    Ok(result)
}
