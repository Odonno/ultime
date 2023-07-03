use leptos::*;

#[server(SignOut, "/api")]
pub async fn sign_out(cx: Scope) -> Result<(), ServerFnError> {
    // TODO : invalidate JWT in SurrealDb?

    let cookie = actix_web::cookie::Cookie::build("access_token", "")
        .path("/")
        .secure(true)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish();

    let response = expect_context::<leptos_actix::ResponseOptions>(cx);
    response.insert_header(
        actix_web::http::header::SET_COOKIE,
        actix_web::http::header::HeaderValue::from_str(&cookie.to_string())
            .map_err(|_| ServerFnError::ServerError("Cannot set cookie".to_string()))?,
    );

    Ok(())
}
