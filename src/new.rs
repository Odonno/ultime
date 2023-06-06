use anyhow::{anyhow, Context, Result};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub enum UltimeProjectTemplate {
    Blog,
}

pub fn main(name: String) -> Result<()> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

    let template = UltimeProjectTemplate::Blog;

    let template_dir_name = get_template_name(template);
    let from = TEMPLATES_DIR
        .get_dir(template_dir_name)
        .context("Cannot get template dir")?;

    let to = name.to_string();
    let to = Path::new(&to);

    if to.exists() {
        return Err(anyhow!("Project already exists"));
    }

    fs::create_dir_all(to)?;

    let result = create_project_dir(from, to, &name);

    if let Err(error) = result {
        fs::remove_dir_all(to)?;
        return Err(error);
    }

    println!("Creating new project: {}", name);

    Ok(())
}

fn create_project_dir(from: &Dir, to: &Path, name: &str) -> Result<()> {
    extract(from, to)?;

    let cargo_toml_jinja2_path = to.join("Cargo.toml.jinja2");
    let cargo_toml_jinja2_content = fs::read_to_string(&cargo_toml_jinja2_path)?;

    let mut env = Environment::new();
    env.add_template("Cargo.toml", &cargo_toml_jinja2_content)?;
    let cargo_toml_template = env.get_template("Cargo.toml")?;

    let cargo_toml_content = cargo_toml_template.render(context! { name })?;
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

    Ok(())
}

fn get_template_name(template: UltimeProjectTemplate) -> &'static str {
    match template {
        UltimeProjectTemplate::Blog => "blog",
    }
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
