use leptos::*;
use serde::{Deserialize, Serialize};

type BlogPostId = String;
type CommentId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommentTarget {
    BlogPost(BlogPostId),
    Comment(CommentId),
}

impl IntoAttribute for CommentTarget {
    fn into_attribute(self, _: Scope) -> Attribute {
        match self {
            CommentTarget::BlogPost(id) => Attribute::String(format!("blog-post-{}", id).into()),
            CommentTarget::Comment(id) => Attribute::String(format!("comment-{}", id).into()),
        }
    }

    fn into_attribute_boxed(self: Box<Self>, cx: Scope) -> Attribute {
        self.into_attribute(cx)
    }
}

#[server(CommentPostOrComment, "/api")]
pub async fn comment(cx: Scope, target: String, content: String) -> Result<(), ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, sql::Thing, Surreal};

    use crate::db::mutations::comment::mutate_comment;

    fn parse_target(target: String) -> CommentTarget {
        let target = target.as_str();
        let target = target.split('-').collect::<Vec<&str>>();
        let target = target.as_slice();

        match target {
            ["blog", "post", id] => CommentTarget::BlogPost(id.to_string()),
            ["comment", id] => CommentTarget::Comment(id.to_string()),
            _ => panic!("Invalid comment target"),
        }
    }

    let target = parse_target(target);

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

    let user_id = "admin";

    let post_id = match &target {
        CommentTarget::BlogPost(id) => Some(id),
        CommentTarget::Comment(_) => None,
    };

    let comment_id = match &target {
        CommentTarget::BlogPost(_) => None,
        CommentTarget::Comment(id) => Some(id),
    };

    let comment = mutate_comment(
        &db,
        user_id,
        post_id.cloned(),
        comment_id.cloned(),
        &content,
    )
    .await
    .map_err(|_| ServerFnError::ServerError("Cannot comment".to_string()))?;

    Ok(())
}
