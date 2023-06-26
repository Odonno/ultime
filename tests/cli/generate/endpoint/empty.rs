use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn generate_new_leptos_endpoint() -> Result<()> {
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
        .arg("fetchBlogPosts");

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

#[server(FetchBlogPosts, "/api")]
pub async fn fetch_blog_posts() -> Result<(), ServerFnError> {
    Ok(())
}"#,
    );

    temp_dir.close()?;

    Ok(())
}
