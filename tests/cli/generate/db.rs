use anyhow::{Context, Result};
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn generate_db_module_from_empty_template() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir).arg("new").arg("ultime-project");

        cmd.assert().success();
    }

    let project_dir = temp_dir.child("ultime-project");

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&project_dir).arg("generate").arg("db");

        cmd.assert().success().stdout("db folder generated...\n");

        let src_dir = project_dir.child("src");
        let db_file = src_dir.child("db.rs");

        assert!(db_file.exists());
        db_file.assert("pub mod crud;");

        let db_dir = src_dir.child("db");
        assert!(db_dir.exists());

        let crud_dir = db_dir.child("crud");
        assert!(crud_dir.exists());

        let crud_files = crud_dir.read_dir()?;
        let crud_files = crud_files.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(crud_files.len(), 1);

        let crud_file = crud_files.first().context("No crud file")?;
        assert_eq!(crud_file.file_name(), "script_migration.rs");
    }

    temp_dir.close()?;

    Ok(())
}

#[test]
fn generate_db_module_from_blog_template() -> Result<()> {
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

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&project_dir).arg("generate").arg("db");

        cmd.assert().success().stdout("db folder generated...\n");

        // TODO : Assert on project_dir
        let src_dir = project_dir.child("src");
        let db_file = src_dir.child("db.rs");

        assert!(db_file.exists());
        db_file.assert(
            "pub mod crud;
pub mod events;
pub mod mutations;
pub mod queries;",
        );

        let db_dir = src_dir.child("db");
        assert!(db_dir.exists());

        {
            let crud_dir = db_dir.child("crud");
            assert!(crud_dir.exists());

            let crud_files = crud_dir.read_dir()?;
            let crud_files = crud_files.collect::<Result<Vec<_>, _>>()?;
            assert_eq!(crud_files.len(), 4);

            let crud_file_names = crud_files.iter().map(|f| f.file_name()).collect::<Vec<_>>();
            assert_eq!(
                crud_file_names,
                vec!["user.rs", "comment.rs", "script_migration.rs", "post.rs"]
            );
        }

        {
            let events_dir = db_dir.child("events");
            assert!(events_dir.exists());

            let events_files = events_dir.read_dir()?;
            let events_files = events_files.collect::<Result<Vec<_>, _>>()?;
            assert_eq!(events_files.len(), 2);

            let events_file_names = events_files
                .iter()
                .map(|f| f.file_name())
                .collect::<Vec<_>>();
            assert_eq!(
                events_file_names,
                vec!["unpublish_post.rs", "publish_post.rs"]
            );
        }

        {
            let mutations_dir = db_dir.child("mutations");
            assert!(mutations_dir.exists());

            let mutations_files = mutations_dir.read_dir()?;
            let mutations_files = mutations_files.collect::<Result<Vec<_>, _>>()?;
            assert_eq!(mutations_files.len(), 1);

            let mutations_file_names = mutations_files
                .iter()
                .map(|f| f.file_name())
                .collect::<Vec<_>>();
            assert_eq!(mutations_file_names, vec!["comment.rs"]);
        }

        {
            let queries_dir = db_dir.child("queries");
            assert!(queries_dir.exists());

            let queries_files = queries_dir.read_dir()?;
            let queries_files = queries_files.collect::<Result<Vec<_>, _>>()?;
            assert_eq!(queries_files.len(), 2);

            let queries_file_names = queries_files
                .iter()
                .map(|f| f.file_name())
                .collect::<Vec<_>>();
            assert_eq!(queries_file_names, vec!["posts.rs", "post_by_id.rs"]);
        }
    }

    temp_dir.close()?;

    Ok(())
}
