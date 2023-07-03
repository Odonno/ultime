use leptos::*;

#[server(SignIn, "/api")]
pub async fn sign_in(cx: Scope, username: String, password: String) -> Result<(), ServerFnError> {
    use serde::Serialize;
    use surrealdb::{
        engine::remote::ws::Ws, opt::auth::Jwt, opt::auth::Scope, sql::Value, Surreal,
    };

    #[derive(Debug, Serialize)]
    struct SignInAuthParams {
        username: String,
        password: String,
    }

    let db = Surreal::new::<Ws>("localhost:8000").await.map_err(|_| {
        ServerFnError::ServerError("Cannot open connection to SurrealDB".to_string())
    })?;

    let token: Jwt = db
        .signin(Scope {
            namespace: "test",
            database: "test",
            scope: "user_scope",
            params: SignInAuthParams { username, password },
        })
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot signin to SurrealDB".to_string()))?;

    // TODO : use "as_insecure_token" fn in beta-10
    let token_string = Value::from(token).as_raw_string();

    let cookie = actix_web::cookie::Cookie::build("access_token", token_string)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();

    let response = expect_context::<leptos_actix::ResponseOptions>(cx);
    response.insert_header(
        actix_web::http::header::SET_COOKIE,
        actix_web::http::header::HeaderValue::from_str(&cookie.to_string())
            .map_err(|_| ServerFnError::ServerError("Cannot set cookie".to_string()))?,
    );

    leptos_actix::redirect(cx, "/");

    Ok(())
}
