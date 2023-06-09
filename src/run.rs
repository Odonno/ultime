use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use surrealdb::sql::{
    statements::{DefineFieldStatement, DefineStatement, DefineTableStatement},
    Statement,
};

enum SurrealType {
    Id,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
struct StructField {
    name: String,
    type_str: String,
}

pub fn main() -> Result<()> {
    if !is_valid_ultime_project() {
        return Err(anyhow!("This is not a valid ultime project"));
    }

    start_surrealdb_instance()?;

    let queries_dir = Path::new("queries");
    if queries_dir.exists() {
        // TODO : Generate queries
    }

    // Generate crud queries
    let mut schemas_to_generate: HashMap<String, String> = HashMap::new();

    let schemas_dir = Path::new("schemas");
    let schemas_files = schemas_dir.read_dir()?;

    for schema_file in schemas_files {
        let schema_file = schema_file?;
        let schema_file_path = schema_file.path();
        let schema_file_name = schema_file_path.file_name().unwrap().to_str().unwrap();
        let schema_file_name_without_extension = schema_file_name.split('.').next().unwrap();
        let schema_file_content = std::fs::read_to_string(&schema_file_path)?;

        println!("schema_file_name: {}", schema_file_name);
        println!(
            "schema_file_name_without_extension: {}",
            schema_file_name_without_extension
        );
        println!("schema_file_content: {}", schema_file_content);

        let parsed_schema = surrealdb::sql::parse(&schema_file_content)?;
        let schema_statements = parsed_schema.0 .0;

        let define_table_statements = extract_define_table_statements(schema_statements.clone());

        let tables = define_table_statements
            .into_iter()
            .map(|define_table_statement| define_table_statement.name.to_string())
            .collect::<Vec<_>>();

        let define_field_statements = extract_define_field_statements(schema_statements);

        for table in tables {
            let table_name = table;
            let struct_name = table_name.to_case(Case::Pascal);

            let mut struct_fields: HashMap<String, SurrealType> = HashMap::new();
            struct_fields.insert("id".to_string(), SurrealType::Id);

            let define_field_statements = define_field_statements
                .clone()
                .into_iter()
                .filter(|define_field_statement| {
                    define_field_statement.what.to_string() == table_name
                })
                .collect::<Vec<_>>();

            for define_field_statement in define_field_statements {
                let field_name = define_field_statement.name.to_string();
                // TODO : Handle other field types
                let field_type = match define_field_statement.kind {
                    _ => SurrealType::Unknown,
                };

                struct_fields.insert(field_name, field_type);
            }

            let struct_fields = struct_fields
                .iter()
                .map(|(field_name, field_type)| {
                    let type_str = match field_type {
                        SurrealType::Id => "Thing",
                        SurrealType::Unknown => "String",
                    };

                    StructField {
                        name: field_name.to_string(),
                        type_str: type_str.to_string(),
                    }
                })
                .collect::<Vec<_>>();

            const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

            let template_content = TEMPLATES_DIR
                .get_file("crud.rs.jinja2")
                .context("Cannot get template 'crud.rs.jinja2'")?
                .contents_utf8()
                .context("Cannot get template 'crud.rs.jinja2'")?
                .to_string();

            let mut env = Environment::new();
            env.add_template("crud.rs", &template_content)?;
            let template = env.get_template("crud.rs")?;

            let content = template.render(context! { table_name, struct_name, struct_fields })?;

            // TODO : get by id
            // TODO : create
            // TODO : update
            // TODO : delete
            // TODO : only get functions for "script_migration"

            schemas_to_generate.insert(table_name, content);
        }
    }

    let src_dir = Path::new("src");

    let has_schemas_to_generate = !schemas_to_generate.is_empty();
    if has_schemas_to_generate {
        let db_dir = src_dir.join("db");
        ensures_folder_exists(&db_dir)?;

        let crud_dir = db_dir.join("crud");
        ensures_folder_exists(&crud_dir)?;

        for (table_name, template) in &schemas_to_generate {
            let generated_schema_file_name = format!("{}.rs", table_name);
            let generated_schema_file_path = crud_dir.join(generated_schema_file_name);

            std::fs::write(generated_schema_file_path, template)?;
        }

        let crud_mod_file_path = crud_dir.join("crud.rs");

        let crud_mod_file_content = schemas_to_generate
            .keys()
            .map(|table_name| format!("pub mod {};", table_name))
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(crud_mod_file_path, crud_mod_file_content)?;
    }

    let events_dir = Path::new("events");
    if events_dir.exists() {
        // TODO : Generate mutations
    }

    let has_db_changes = has_schemas_to_generate;
    if has_db_changes {
        let mod_file_path = src_dir.join("db.rs");

        let mod_file_content = if has_schemas_to_generate {
            "pub mod crud;"
        } else {
            ""
        };

        std::fs::write(mod_file_path, mod_file_content)?;
    }

    start_leptos_app()?;

    // TODO : Apply watch mode to regenerate queries/mutations/crud on each change

    Ok(())
}

fn is_valid_ultime_project() -> bool {
    let has_cargo_toml = Path::new("Cargo.toml").exists();
    has_cargo_toml
}

fn start_surrealdb_instance() -> Result<()> {
    let check_surreal_cli = Command::new("surreal")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    if check_surreal_cli.success() {
        println!("Start a new SurrealDB instance...");

        let username = "test";
        let password = "test";

        let start_surrealdb_instance = Command::new("surreal")
            .arg("start")
            .arg("--user")
            .arg(username)
            .arg("--pass")
            .arg(password)
            .arg("memory")
            .spawn()?
            .wait()?;

        if start_surrealdb_instance.success() {
            println!("SurrealDB instance started successfully");
        } else {
            return Err(anyhow!("SurrealDB instance failed to start"));
        }

        // TODO : Try to apply migrations on the SurrealDB instance
    } else {
        println!("surreal cli does not seem to be installed. Step skipped...");
    }

    Ok(())
}

fn start_leptos_app() -> Result<()> {
    println!("Start leptos app...");
    let _start_leptos_app = Command::new("cargo").arg("leptos").arg("watch").spawn()?;

    // TODO : Handle error if port (3000) already in use
    // TODO : Use a different port if 3000 is already in use

    Ok(())
}

fn extract_define_table_statements(statements: Vec<Statement>) -> Vec<DefineTableStatement> {
    statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Define(define_statement) => Some(define_statement),
            _ => None,
        })
        .filter_map(|define_statement| match define_statement {
            DefineStatement::Table(define_table_statement) => Some(define_table_statement),
            _ => None,
        })
        .collect::<Vec<_>>()
}

fn extract_define_field_statements(statements: Vec<Statement>) -> Vec<DefineFieldStatement> {
    statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Define(define_statement) => Some(define_statement),
            _ => None,
        })
        .filter_map(|define_statement| match define_statement {
            DefineStatement::Field(define_field_statement) => Some(define_field_statement),
            _ => None,
        })
        .collect::<Vec<_>>()
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}
