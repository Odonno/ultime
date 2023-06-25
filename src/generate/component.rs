use anyhow::{Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use std::path::{Path, PathBuf};

pub fn main(name: String) -> Result<()> {
    let src_dir = Path::new("src");

    let components_dir = src_dir.join("components");
    ensures_folder_exists(&components_dir)?;

    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file("component.rs.jinja2")
        .context("Cannot get template 'component.rs.jinja2'")?
        .contents_utf8()
        .context("Cannot get template 'component.rs.jinja2'")?
        .to_string();

    let component_name = name.to_case(Case::Pascal);
    let content = Environment::new().render_str(&template_content, context! { component_name })?;

    let file_name = name.to_case(Case::Snake);
    let file_name = format!("{}.rs", file_name);

    let component_file = components_dir.join(file_name);

    std::fs::write(component_file, content)?;

    println!("Component {} successfully created", name);

    Ok(())
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}
