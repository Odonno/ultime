use leptos::*;

use crate::db::crud::post::Post;

#[server(CreateBlogPost, "/api")]
pub async fn create_blog_post(
    cx: Scope,
    title: String,
    content: String,
) -> Result<Post, ServerFnError> {
    use serde::{Deserialize, Serialize};
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

    #[derive(Serialize, Deserialize)]
    struct CreateBlogPostContent {
        title: String,
        content: String,
        author: Thing,
    }

    let db = Surreal::new::<Ws>("localhost:8000").await.map_err(|_| {
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

    // TODO : use crate::db::crud::post::create_post function
    let post: Post = db
        .create("post")
        .content(CreateBlogPostContent {
            title: title.to_string(),
            content: content.to_string(),
            author: ("user", "admin").into(),
        })
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot create post".to_string()))?;

    leptos_actix::redirect(cx, format!("/posts/{}", post.id.id).as_str());

    Ok(post)
}
