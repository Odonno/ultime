use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};

use crate::helpers::*;

#[test]
fn generate_new_leptos_component() -> Result<()> {
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
        .arg("page")
        .arg("home");

    cmd.assert()
        .success()
        .stdout("Page home successfully created\n");

    let src_dir = project_dir.child("src");
    let pages_dir = src_dir.child("pages");

    assert!(pages_dir.exists());

    let home_page_file = pages_dir.child("home.rs");

    assert!(home_page_file.is_file());
    home_page_file.assert(
        r#"use leptos::*;

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let (count, set_count) = create_signal(cx, 0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <button type="button" on:click=on_click>
            {count}
        </button>
    }
}"#,
    );

    temp_dir.close()?;

    Ok(())
}
