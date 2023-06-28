use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn fails_to_generate_endpoint_if_schema_does_not_exist() -> Result<()> {
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
        .arg("--from-schema")
        .arg("non-existing-schema");

    cmd.assert()
        .failure()
        .stderr("Error: Schema 'non-existing-schema.surql' does not exist\n");

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_list_method() -> Result<()> {
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
        .arg("listPosts")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("list");

    cmd.assert()
        .success()
        .stdout("Endpoint listPosts successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("list_posts.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(ListPosts, "/api")]
pub async fn list_posts() -> Result<Vec<Post>, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::get_all_post;

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

    get_all_post(&db)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function get_all_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_get_method() -> Result<()> {
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
        .arg("getPost")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("get");

    cmd.assert()
        .success()
        .stdout("Endpoint getPost successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("get_post.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(GetPost, "/api")]
pub async fn get_post(
    id: &'static str,
) -> Result<Post, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::get_post;

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

    get_post(&db, id)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function get_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_find_method() -> Result<()> {
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
        .arg("findPost")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("find");

    cmd.assert()
        .success()
        .stdout("Endpoint findPost successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("find_post.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(FindPost, "/api")]
pub async fn find_post(
    id: &'static str,
) -> Result<Option<Post>, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::find_post;

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

    find_post(&db, id)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function find_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_create_method() -> Result<()> {
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
        .arg("createPost")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("create");

    cmd.assert()
        .success()
        .stdout("Endpoint createPost successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("create_post.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(CreatePost, "/api")]
pub async fn create_post(
    data: Post,
) -> Result<Post, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::create_post;

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

    create_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function create_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_update_method() -> Result<()> {
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
        .arg("updatePost")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("update");

    cmd.assert()
        .success()
        .stdout("Endpoint updatePost successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("update_post.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(UpdatePost, "/api")]
pub async fn update_post(
    data: Post,
) -> Result<Post, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::update_post;

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

    update_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function update_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_delete_method() -> Result<()> {
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
        .arg("deletePost")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("delete");

    cmd.assert()
        .success()
        .stdout("Endpoint deletePost successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("delete_post.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(DeletePost, "/api")]
pub async fn delete_post(
    data: Post,
) -> Result<Post, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::delete_post;

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

    delete_post(&db, data)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function delete_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_new_leptos_endpoint_from_schema_with_delete_all_method() -> Result<()> {
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
        .arg("deleteAllPosts")
        .arg("--from-schema")
        .arg("post")
        .arg("--method")
        .arg("delete-all");

    cmd.assert()
        .success()
        .stdout("Endpoint deleteAllPosts successfully created\n");

    let src_dir = project_dir.child("src");
    let api_dir = src_dir.child("api");

    assert!(api_dir.exists());

    let endpoint_file = api_dir.child("delete_all_posts.rs");

    assert!(endpoint_file.is_file());
    endpoint_file.assert(
        r#"use leptos::*;

use crate::db::crud::post::Post;

#[server(DeleteAllPosts, "/api")]
pub async fn delete_all_posts() -> Result<Vec<Post>, ServerFnError> {
    use surrealdb::{engine::remote::ws::Ws, Surreal, opt::auth::Root};

    use crate::db::crud::post::delete_all_post;

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

    delete_all_post(&db)
        .await
        .map_err(|_| ServerFnError::ServerError("Cannot apply function delete_all_post".to_string()))?;

    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}
