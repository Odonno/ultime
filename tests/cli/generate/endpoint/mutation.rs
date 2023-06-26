use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn fails_to_generate_endpoint_if_mutation_does_not_exist() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir).arg("new").arg("ultime-project");

        cmd.assert().success();
    }

    let project_dir = temp_dir.child("ultime-project");

    let mut cmd = create_cmd()?;
    cmd.current_dir(&project_dir)
        .arg("generate")
        .arg("endpoint")
        .arg("comment")
        .arg("--from-mutation")
        .arg("non-existing-mutation");

    cmd.assert()
        .failure()
        .stderr("Error: Mutation 'non-existing-mutation.surql' does not exist\n");

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_mutation() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir)
            .arg("new")
            .arg("my-blog")
            .arg("--template")
            .arg("blog");

        cmd.assert().success();
    }

    let project_dir = temp_dir.child("my-blog");

    let mut cmd = create_cmd()?;
    cmd.current_dir(&project_dir)
        .arg("generate")
        .arg("endpoint")
        .arg("comment")
        .arg("--from-mutation")
        .arg("comment");

    cmd.assert()
        .success()
        .stdout("Endpoint comment successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("comment.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::models::mutations::CommentMutation;

#[server(Comment, "/api")]
pub async fn comment(
    user_id: String,
    post_id: Option<String>,
    comment_id: Option<String>,
    content: String,
) -> Result<CommentMutation, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::mutations::comment::mutate_comment;

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

    let result = mutate_comment(&db, user_id, post_id, comment_id, content)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply mutation comment".to_string()))?;

    Ok(result)
}"#,
    );

    temp_dir.close()?;

    Ok(())
}
