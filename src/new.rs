use anyhow::{anyhow, Context, Result};
use chrono::Local;
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use regex::Regex;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::cli::UltimeProjectTemplate;

pub fn main(name: String, template: Option<UltimeProjectTemplate>) -> Result<()> {
    let template = match template {
        Some(template) => template,
        None => UltimeProjectTemplate::Empty,
    };

    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/projects");

    let template_dir_name = get_template_name(template);
    let from = TEMPLATES_DIR
        .get_dir(template_dir_name)
        .context("Cannot get template dir")?;

    let to = Path::new(&name);

    if to.exists() {
        return Err(anyhow!("Project already exists"));
    }

    fs::create_dir_all(to)?;

    let result = create_project_dir(from, to, &name);

    if let Err(error) = result {
        fs::remove_dir_all(to)?;
        return Err(error);
    }

    println!("Project '{}' created. Run the following command:", name);
    println!("cd {}", name);

    Ok(())
}

fn get_template_name(template: UltimeProjectTemplate) -> &'static str {
    match template {
        UltimeProjectTemplate::Empty => "empty",
        UltimeProjectTemplate::Blog => "blog",
    }
}

fn create_project_dir(from: &Dir, to: &Path, name: &str) -> Result<()> {
    let now = Local::now();

    {
        println!("Cloning template...");
        extract(from, to)?;
    }

    {
        let cargo_toml_jinja2_path = to.join("Cargo.toml.jinja2");
        let cargo_toml_jinja2_content = fs::read_to_string(&cargo_toml_jinja2_path)?;

        let cargo_toml_content =
            Environment::new().render_str(&cargo_toml_jinja2_content, context! { name })?;
        let cargo_toml_path = to.join("Cargo.toml");

        // Create file
        let mut fsf = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&cargo_toml_path)?;
        fsf.write_all(cargo_toml_content.as_bytes())?;
        fsf.sync_all()?;

        // Remove jinja2 file
        fs::remove_file(cargo_toml_jinja2_path)?;
    }

    {
        let index_html_jinja2_path = to.join("index.html.jinja2");
        let index_html_jinja2_content = fs::read_to_string(&index_html_jinja2_path)?;

        let index_html_content =
            Environment::new().render_str(&index_html_jinja2_content, context! { name })?;
        let index_html_path = to.join("index.html");

        // Create file
        let mut fsf = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&index_html_path)?;
        fsf.write_all(index_html_content.as_bytes())?;
        fsf.sync_all()?;

        // Remove jinja2 file
        fs::remove_file(index_html_jinja2_path)?;
    }

    let src_folder = to.join("src");

    {
        let app_rs_jinja2_path = src_folder.join("app.rs.jinja2");
        let app_rs_jinja2_content = fs::read_to_string(&app_rs_jinja2_path)?;

        let app_rs_content =
            Environment::new().render_str(&app_rs_jinja2_content, context! { name })?;
        let app_rs_path = src_folder.join("app.rs");

        // Create file
        let mut fsf = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&app_rs_path)?;
        fsf.write_all(app_rs_content.as_bytes())?;
        fsf.sync_all()?;

        // Remove jinja2 file
        fs::remove_file(app_rs_jinja2_path)?;
    }

    {
        let name = name.replace('-', "_");

        let main_rs_jinja2_path = src_folder.join("main.rs.jinja2");
        let main_rs_jinja2_content = fs::read_to_string(&main_rs_jinja2_path)?;

        let main_rs_content =
            Environment::new().render_str(&main_rs_jinja2_content, context! { name })?;
        let main_rs_path = src_folder.join("main.rs");

        // Create file
        let mut fsf = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&main_rs_path)?;
        fsf.write_all(main_rs_content.as_bytes())?;
        fsf.sync_all()?;

        // Remove jinja2 file
        fs::remove_file(main_rs_jinja2_path)?;
    }

    {
        println!("Creating migration project...");

        // Rename files in migrations folder
        let migrations_dir_path = to.join("migrations");
        if migrations_dir_path.exists() {
            let regex = Regex::new(r"^YYYYMMDD_HHMM(\d{2})_")?;

            let migrations_dir = fs::read_dir(&migrations_dir_path)?;

            let migration_filenames_to_rename = migrations_dir
                .filter_map(|entry| match entry {
                    Ok(file) => {
                        let file_name = file.file_name();
                        if regex.is_match(file_name.to_str().unwrap_or("")) {
                            Some(file_name)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                })
                .collect::<Vec<_>>();

            for filename in migration_filenames_to_rename {
                let filename = filename
                    .to_str()
                    .context("Cannot convert filename to string")?;

                let captures = regex
                    .captures(filename)
                    .context("Cannot retrieve from pattern")?;
                let seconds = captures
                    .get(1)
                    .context("Cannot retrieve from pattern")?
                    .as_str();

                let new_filename_prefix = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);
                let new_filename = regex.replace(filename, new_filename_prefix);

                let from = format!("{}/{}", migrations_dir_path.display(), filename);
                let to = format!("{}/{}", migrations_dir_path.display(), new_filename);

                fs::rename(from, to)?;
            }
        }

        // Rename files in migrations/down folder
        let down_migrations_dir_path = migrations_dir_path.join("down");
        if down_migrations_dir_path.exists() {
            let regex = Regex::new(r"^YYYYMMDD_HHMM(\d{2})_")?;

            let down_migrations_dir = fs::read_dir(&down_migrations_dir_path)?;

            let down_migration_filenames_to_rename = down_migrations_dir
                .filter_map(|entry| match entry {
                    Ok(file) => {
                        let file_name = file.file_name();
                        if regex.is_match(file_name.to_str().unwrap_or("")) {
                            Some(file_name)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                })
                .collect::<Vec<_>>();

            for filename in down_migration_filenames_to_rename {
                let filename = filename
                    .to_str()
                    .context("Cannot convert filename to string")?;

                let captures = regex
                    .captures(filename)
                    .context("Cannot retrieve from pattern")?;
                let seconds = captures
                    .get(1)
                    .context("Cannot retrieve from pattern")?
                    .as_str();

                let new_filename_prefix = format!("{}{}_", now.format("%Y%m%d_%H%M"), seconds);
                let new_filename = regex.replace(filename, new_filename_prefix);

                let from = format!("{}/{}", down_migrations_dir_path.display(), filename);
                let to = format!("{}/{}", down_migrations_dir_path.display(), new_filename);

                fs::rename(from, to)?;
            }
        }
    }

    Ok(())
}

// 💡 Function extract customized because it is not implemented in the "include_dir" crate.
// cf. https://github.com/Michael-F-Bryan/include_dir/pull/60
pub fn extract<S: AsRef<Path>>(dir: &Dir<'_>, path: S) -> std::io::Result<()> {
    fn extract_dir<S: AsRef<Path>>(dir: Dir<'_>, path: S) -> std::io::Result<()> {
        let path = path.as_ref();

        for dir in dir.dirs() {
            let dir_path = dir.path().components().skip(1).collect::<PathBuf>();

            std::fs::create_dir_all(path.join(dir_path))?;
            extract_dir(dir.clone(), path)?;
        }

        for file in dir.files() {
            let file_path = file.path().components().skip(1).collect::<PathBuf>();

            let mut fsf = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(file_path))?;
            fsf.write_all(file.contents())?;
            fsf.sync_all()?;
        }

        Ok(())
    }

    extract_dir(dir.clone(), path)
}
