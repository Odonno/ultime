use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use notify::{
    event::{AccessKind, AccessMode},
    EventKind, RecursiveMode, Watcher,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use surrealdb::sql::{
    statements::{
        DefineEventStatement, DefineFieldStatement, DefineStatement, DefineTableStatement,
    },
    Kind, Statement,
};

enum SurrealType {
    Id,
    String,
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
    generate_db_folder()?;
    start_leptos_app()?;
    watch_to_regenerate_db_folder()?;

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
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

fn generate_db_folder() -> Result<()> {
    let src_dir = Path::new("src");

    // TODO : Generate queries
    let _queries_dir = Path::new("queries");

    // Generate crud queries
    let mut schemas_to_generate: HashMap<String, String> = HashMap::new();
    let mut has_schemas_to_generate = false;

    let schemas_dir = Path::new("schemas");
    if schemas_dir.exists() {
        let schemas_files = schemas_dir.read_dir()?;

        for schema_file in schemas_files {
            let schema_file = schema_file?;
            let schema_file_path = schema_file.path();
            let schema_file_content = std::fs::read_to_string(&schema_file_path)?;

            let parsed_schema = surrealdb::sql::parse(&schema_file_content)?;
            let schema_statements = parsed_schema.0 .0;

            let define_table_statements =
                extract_define_table_statements(schema_statements.clone());
            let define_field_statements = extract_define_field_statements(schema_statements);

            let tables = define_table_statements
                .into_iter()
                .map(|define_table_statement| define_table_statement.name.to_string())
                .collect::<Vec<_>>();

            for table in tables {
                let table_name = table;
                let struct_name = table_name.to_case(Case::Pascal);

                let define_field_statements = define_field_statements
                    .clone()
                    .into_iter()
                    .filter(|define_field_statement| {
                        define_field_statement.what.to_string() == table_name
                    })
                    .collect::<Vec<_>>();

                let struct_fields = extract_struct_fields(define_field_statements);

                let content = generate_from_crud_template(
                    table_name.to_string(),
                    struct_name,
                    struct_fields,
                )?;

                schemas_to_generate.insert(table_name, content);
            }
        }

        has_schemas_to_generate = !schemas_to_generate.is_empty();
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
    }

    // Generate mutations
    let mut events_to_generate: HashMap<String, String> = HashMap::new();
    let mut has_events_to_generate = false;

    let events_dir = Path::new("events");
    if events_dir.exists() {
        let events_files = events_dir.read_dir()?;

        for event_files in events_files {
            let event_files = event_files?;
            let event_files_path = event_files.path();
            let event_files_content = std::fs::read_to_string(&event_files_path)?;

            let parsed_event = surrealdb::sql::parse(&event_files_content)?;
            let event_statements = parsed_event.0 .0;

            let define_table_statements = extract_define_table_statements(event_statements.clone());
            let define_field_statements = extract_define_field_statements(event_statements.clone());
            let define_event_statements = extract_define_event_statements(event_statements);

            let events = define_event_statements
                .into_iter()
                .map(|define_event_statement| define_event_statement.name.to_string())
                .collect::<Vec<_>>();

            for event in events {
                let table_name = event;

                let table_definition_statement =
                    define_table_statements
                        .clone()
                        .into_iter()
                        .find(|define_table_statement| {
                            define_table_statement.name.to_string() == table_name
                        });

                if table_definition_statement.is_none() {
                    continue;
                }

                let struct_name = format!("{}{}", table_name, "Data").to_case(Case::Pascal);
                let func_name = table_name.to_case(Case::Snake);

                let define_field_statements = define_field_statements
                    .clone()
                    .into_iter()
                    .filter(|define_field_statement| {
                        define_field_statement.what.to_string() == table_name
                    })
                    .collect::<Vec<_>>();

                let struct_fields = extract_struct_fields(define_field_statements);

                let content = generate_from_mutation_template(
                    func_name,
                    table_name.to_string(),
                    struct_name,
                    struct_fields,
                )?;

                events_to_generate.insert(table_name, content);
            }
        }

        has_events_to_generate = !events_to_generate.is_empty();
        if has_events_to_generate {
            let db_dir = src_dir.join("db");
            ensures_folder_exists(&db_dir)?;

            let events_dir = db_dir.join("events");
            ensures_folder_exists(&events_dir)?;

            for (table_name, template) in &events_to_generate {
                let generated_events_file_name = format!("{}.rs", table_name);
                let generated_events_file_path = events_dir.join(generated_events_file_name);

                std::fs::write(generated_events_file_path, template)?;
            }

            let events_mod_file_path = events_dir.join("events.rs");

            let events_mod_file_content = events_to_generate
                .keys()
                .map(|table_name| format!("pub mod {};", table_name))
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(events_mod_file_path, events_mod_file_content)?;
        }
    }

    // Generate db.rs
    let has_db_changes = has_schemas_to_generate || has_events_to_generate;
    if has_db_changes {
        let mod_file_path = src_dir.join("db.rs");
        let mut mod_file_modules = vec![];

        if has_schemas_to_generate {
            mod_file_modules.push("crud");
        }
        if has_events_to_generate {
            mod_file_modules.push("events");
        }

        let mod_file_content = mod_file_modules
            .iter()
            .map(|module| format!("pub mod {};", module))
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(mod_file_path, mod_file_content)?;
    } else {
        // TODO : Remove db.rs if it exists
    }

    Ok(())
}

fn extract_struct_fields(define_field_statements: Vec<DefineFieldStatement>) -> Vec<StructField> {
    let mut struct_fields: HashMap<String, SurrealType> = HashMap::new();
    struct_fields.insert("id".to_string(), SurrealType::Id);

    for define_field_statement in define_field_statements {
        let field_name = define_field_statement.name.to_string();
        // TODO : Handle other field types
        let field_type = match define_field_statement.kind {
            Some(Kind::String) => SurrealType::String,
            _ => SurrealType::Unknown,
        };

        struct_fields.insert(field_name, field_type);
    }

    let struct_fields = struct_fields
        .iter()
        .map(|(field_name, field_type)| {
            let type_str = match field_type {
                SurrealType::Id => "Thing",
                SurrealType::String => "String",
                SurrealType::Unknown => "String", // TODO : What to do here?
            };

            StructField {
                name: field_name.to_string(),
                type_str: type_str.to_string(),
            }
        })
        .collect::<Vec<_>>();
    struct_fields
}

fn watch_to_regenerate_db_folder() -> Result<()> {
    fn watch_event(result: notify::Result<notify::Event>) {
        match result {
            Ok(event) => {
                if let EventKind::Access(AccessKind::Close(AccessMode::Write)) = event.kind {
                    let _paths = event.paths;
                    let result = generate_db_folder();

                    match result {
                        Ok(_) => {
                            println!("db folder generated...");
                        }
                        Err(error) => {
                            eprintln!("Error while generating db folder: {:?}", error);
                        }
                    }
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }

    let mut watcher = notify::recommended_watcher(watch_event)?;

    let schemas_dir = Path::new("schemas");
    if schemas_dir.exists() {
        watcher.watch(schemas_dir, RecursiveMode::NonRecursive)?;
    }

    let events_dir = Path::new("events");
    if events_dir.exists() {
        watcher.watch(events_dir, RecursiveMode::NonRecursive)?;
    }

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

fn extract_define_event_statements(statements: Vec<Statement>) -> Vec<DefineEventStatement> {
    statements
        .into_iter()
        .filter_map(|statement| match statement {
            Statement::Define(define_statement) => Some(define_statement),
            _ => None,
        })
        .filter_map(|define_statement| match define_statement {
            DefineStatement::Event(define_event_statement) => Some(define_event_statement),
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

fn generate_from_crud_template(
    table_name: String,
    struct_name: String,
    struct_fields: Vec<StructField>,
) -> Result<String> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file("crud.rs.jinja2")
        .context("Cannot get template 'crud.rs.jinja2'")?
        .contents_utf8()
        .context("Cannot get template 'crud.rs.jinja2'")?
        .to_string();

    let content = Environment::new().render_str(
        &template_content,
        context! { table_name, struct_name, struct_fields },
    )?;

    Ok(content)
}

fn generate_from_mutation_template(
    func_name: String,
    table_name: String,
    struct_name: String,
    struct_fields: Vec<StructField>,
) -> Result<String> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file("mutation.rs.jinja2")
        .context("Cannot get template 'mutation.rs.jinja2'")?
        .contents_utf8()
        .context("Cannot get template 'mutation.rs.jinja2'")?
        .to_string();

    let content = Environment::new().render_str(
        &template_content,
        context! { func_name, table_name, struct_name, struct_fields },
    )?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn generate_post_crud_content() {
        let table_name = "post";
        let struct_name = "Post";
        let struct_fields = vec![
            StructField {
                name: "id".to_string(),
                type_str: "Thing".to_string(),
            },
            StructField {
                name: "title".to_string(),
                type_str: "String".to_string(),
            },
            StructField {
                name: "content".to_string(),
                type_str: "String".to_string(),
            },
        ];

        let result = generate_from_crud_template(
            table_name.to_string(),
            struct_name.to_string(),
            struct_fields,
        )
        .unwrap();

        assert_eq!(
            result,
            "use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Connection, Result, Surreal};

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    id: Thing,
    title: String,
    content: String,
}

pub async fn get_all_post<C: Connection>(db: &'_ Surreal<C>) -> Result<Vec<Post>> {
    let result = db.select(\"post\").await?;
    Ok(result)
}

pub async fn get_post<C: Connection>(db: &'_ Surreal<C>, id: &str) -> Result<Post> {
    let result = db.select((\"post\", id)).await?;
    Ok(result)
}

pub async fn find_post<C: Connection>(db: &'_ Surreal<C>, id: &str) -> Result<Option<Post>> {
    let result = db.select((\"post\", id)).await?;
    Ok(result)
}

pub async fn create_post<C: Connection>(db: &'_ Surreal<C>, data: Post) -> Result<Post> {
    let result = db.create(\"post\").content(data).await?;
    Ok(result)
}

pub async fn update_post<C: Connection>(db: &'_ Surreal<C>, id: &str, data: Post) -> Result<Option<Post>> {
    let result = db.update((\"post\", id)).content(data).await?;
    Ok(result)
}

pub async fn delete_all_post<C: Connection>(db: &'_ Surreal<C>) -> Result<Vec<Post>> {
    let result = db.delete(\"post\").await?;
    Ok(result)
}

pub async fn delete_post<C: Connection>(db: &'_ Surreal<C>, id: &str) -> Result<Option<Post>> {
    let result = db.delete((\"post\", id)).await?;
    Ok(result)
}"
        );
    }

    #[test]
    fn generate_script_migration_crud_content() {
        let table_name = "script_migration";
        let struct_name = "ScriptMigration";
        let struct_fields = vec![
            StructField {
                name: "id".to_string(),
                type_str: "Thing".to_string(),
            },
            StructField {
                name: "script_name".to_string(),
                type_str: "String".to_string(),
            },
            StructField {
                name: "executed_at".to_string(),
                type_str: "String".to_string(),
            },
        ];

        let result = generate_from_crud_template(
            table_name.to_string(),
            struct_name.to_string(),
            struct_fields,
        )
        .unwrap();

        assert_eq!(
            result,
            "use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Connection, Result, Surreal};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScriptMigration {
    id: Thing,
    script_name: String,
    executed_at: String,
}

pub async fn get_all_script_migration<C: Connection>(db: &'_ Surreal<C>) -> Result<Vec<ScriptMigration>> {
    let result = db.select(\"script_migration\").await?;
    Ok(result)
}

pub async fn get_script_migration<C: Connection>(db: &'_ Surreal<C>, id: &str) -> Result<ScriptMigration> {
    let result = db.select((\"script_migration\", id)).await?;
    Ok(result)
}

pub async fn find_script_migration<C: Connection>(db: &'_ Surreal<C>, id: &str) -> Result<Option<ScriptMigration>> {
    let result = db.select((\"script_migration\", id)).await?;
    Ok(result)
}"
        );
    }

    #[test]
    fn generate_publish_post_mutation_content() {
        let func_name = "publish_post";
        let table_name = "publish_post";
        let struct_name = "PublishPostData";
        let struct_fields = vec![
            StructField {
                name: "id".to_string(),
                type_str: "Thing".to_string(),
            },
            StructField {
                name: "title".to_string(),
                type_str: "String".to_string(),
            },
            StructField {
                name: "content".to_string(),
                type_str: "String".to_string(),
            },
        ];

        let result = generate_from_mutation_template(
            func_name.to_string(),
            table_name.to_string(),
            struct_name.to_string(),
            struct_fields,
        )
        .unwrap();

        assert_eq!(
            result,
            "use serde::{Deserialize, Serialize};
use surrealdb::{sql::Thing, Connection, Result, Surreal};

#[derive(Serialize, Deserialize, Debug)]
pub struct PublishPostData {
    id: Thing,
    title: String,
    content: String,
}

pub async fn publish_post<C: Connection>(db: &'_ Surreal<C>, data: PublishPostData) -> Result<PublishPostData> {
    let record: PublishPostData = db.create(\"publish_post\").content(data).await?;
    Ok(record)
}"
        );
    }
}
