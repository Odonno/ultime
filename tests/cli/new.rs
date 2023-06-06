use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn create_new_project_with_blog_template() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    let mut cmd = create_cmd()?;
    cmd.current_dir(&temp_dir).arg("new").arg("my-blog");

    cmd.assert()
        .success()
        .stdout("Creating new project: my-blog\n");

    let my_blog_folder = temp_dir.child("my-blog");
    assert!(my_blog_folder.is_dir(), "my-blog dir should exists");

    let cargo_toml_file = my_blog_folder.child("Cargo.toml");
    assert!(cargo_toml_file.is_file(), "Cargo.toml file should exists");
    cargo_toml_file.assert(
        r#"[package]
name = "my-blog"
version = "0.0.1"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
leptos = "0.3.0""#,
    );

    temp_dir.close()?;

    Ok(())
}

#[test]
fn fails_to_create_new_project_if_folder_already_exist() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir).arg("new").arg("my-blog");

        cmd.assert().success();
    }

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir).arg("new").arg("my-blog");

        cmd.assert()
            .failure()
            .stderr("Error: Project already exists\n");
    }

    temp_dir.close()?;

    Ok(())
}
