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
        .arg("component")
        .arg("my-component");

    cmd.assert()
        .success()
        .stdout("Component my-component successfully created\n");

    let src_dir = project_dir.child("src");
    let components_dir = src_dir.child("components");

    assert!(components_dir.exists());

    let my_component_file = components_dir.child("my_component.rs");

    assert!(my_component_file.is_file());
    my_component_file.assert(
        r#"use leptos::*;

#[component]
pub fn MyComponent(cx: Scope) -> impl IntoView {
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
