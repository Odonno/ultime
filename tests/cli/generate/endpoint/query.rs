use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn fails_to_generate_endpoint_if_query_does_not_exist() -> Result<()> {
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
        .arg("fetchBlogPosts")
        .arg("--from-query")
        .arg("non-existing-query");

    cmd.assert()
        .failure()
        .stderr("Error: Query 'non-existing-query.surql' does not exist\n");

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_query() -> Result<()> {
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
        .arg("fetchBlogPosts")
        .arg("--from-query")
        .arg("posts");

    cmd.assert()
        .success()
        .stdout("Endpoint fetchBlogPosts successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("fetch_blog_posts.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::models::queries::PostsQuery;

#[server(FetchBlogPosts, "/api")]
pub async fn fetch_blog_posts() -> Result<PostsQuery, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::queries::posts::query_posts;

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

    let result = query_posts(&db)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply query posts".to_string()))?;

    Ok(result)
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_query_with_params() -> Result<()> {
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
        .arg("fetchPostDetails")
        .arg("--from-query")
        .arg("post_by_id");

    cmd.assert()
        .success()
        .stdout("Endpoint fetchPostDetails successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("fetch_post_details.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::models::queries::PostByIdQuery;

#[server(FetchPostDetails, "/api")]
pub async fn fetch_post_details(
    post_id: String,
) -> Result<PostByIdQuery, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::queries::post_by_id::query_post_by_id;

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

    let result = query_post_by_id(&db, post_id)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply query post_by_id".to_string()))?;

    Ok(result)
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_query_using_rs_file_extension() -> Result<()> {
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
        .arg("fetchBlogPosts")
        .arg("--from-query")
        .arg("posts.rs");

    cmd.assert()
        .success()
        .stdout("Endpoint fetchBlogPosts successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("fetch_blog_posts.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::models::queries::PostsQuery;

#[server(FetchBlogPosts, "/api")]
pub async fn fetch_blog_posts() -> Result<PostsQuery, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::queries::posts::query_posts;

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

    let result = query_posts(&db)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply query posts".to_string()))?;

    Ok(result)
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_query_using_surql_file_extension() -> Result<()> {
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
        .arg("fetchBlogPosts")
        .arg("--from-query")
        .arg("posts.surql");

    cmd.assert()
        .success()
        .stdout("Endpoint fetchBlogPosts successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("fetch_blog_posts.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::models::queries::PostsQuery;

#[server(FetchBlogPosts, "/api")]
pub async fn fetch_blog_posts() -> Result<PostsQuery, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::queries::posts::query_posts;

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

    let result = query_posts(&db)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply query posts".to_string()))?;

    Ok(result)
}"#,
    );

    temp_dir.close()?;

    Ok(())
}
