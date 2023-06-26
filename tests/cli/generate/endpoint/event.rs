use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn fails_to_generate_endpoint_if_event_does_not_exist() -> Result<()> {
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
        .arg("publishPost")
        .arg("--from-event")
        .arg("non-existing-event");

    cmd.assert()
        .failure()
        .stderr("Error: Event 'non-existing-event.surql' does not exist\n");

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_event() -> Result<()> {
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
        .arg("publishPost")
        .arg("--from-event")
        .arg("publish_post");

    cmd.assert()
        .success()
        .stdout("Endpoint publishPost successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("publish_post.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::events::publish_post::PublishPostData;

#[server(PublishPost, "/api")]
pub async fn publish_post(
    data: PublishPostData,
) -> Result<(), ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

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

    crate::db::events::publish_post::publish_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply event publish_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}
