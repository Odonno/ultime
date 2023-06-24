use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use notify::{
    event::{AccessKind, AccessMode},
    EventKind, INotifyWatcher, RecursiveMode, Watcher,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use surrealdb::{
    engine::remote::ws::Ws,
    opt::auth::Root,
    sql::{
        statements::{
            DefineEventStatement, DefineFieldStatement, DefineStatement, DefineTableStatement,
        },
        Kind, Statement,
    },
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

pub async fn main() -> Result<()> {
    if !is_valid_ultime_project() {
        return Err(anyhow!("This is not a valid ultime project"));
    }

    start_surrealdb_instance().await?;
    generate_db_folder()?;
    start_leptos_app()?;
    let _watcher = watch_to_regenerate_db_folder()?; // 💡 prevent watcher to be dropped

    // 💡 infinite loop to keep the process alive
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn is_valid_ultime_project() -> bool {
    let has_cargo_toml = Path::new("Cargo.toml").exists();
    has_cargo_toml
}

async fn start_surrealdb_instance() -> Result<()> {
    let check_surreal_cli = Command::new("surreal")
        .arg("-h")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;

    if check_surreal_cli.success() {
        println!("Start a new SurrealDB instance...");

        let username = "root";
        let password = "root";

        let _start_surreal_instance = Command::new("surreal")
            .arg("start")
            .arg("--user")
            .arg(username)
            .arg("--pass")
            .arg(password)
            .arg("memory")
            .spawn()?;

        // TODO : Read error from child process

        // TODO : Wait until SurrealDB instance is ready
        println!("SurrealDB instance started successfully");

        std::thread::sleep(Duration::from_secs(1)); // TODO : Wait until SurrealDB instance is ready

        // Try to apply migrations on the SurrealDB instance
        let db = surrealdb::Surreal::new::<Ws>("localhost:8000").await?;

        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        db.use_ns("test").use_db("test").await?;

        println!("Start to apply SurrealDB migrations...");

        let result = surrealdb_migrations::MigrationRunner::new(&db).up().await;

        match result {
            Ok(_) => println!("SurrealDB migrations applied successfully"),
            Err(err) => println!("SurrealDB migrations failed to apply: {:?}", err),
        }
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

    // Generate queries
    let mut queries_to_generate: HashMap<String, String> = HashMap::new();
    let mut has_queries_to_generate = false;

    let queries_dir = Path::new("queries");
    if queries_dir.exists() {
        let queries_files = queries_dir.read_dir()?;

        for query_file in queries_files {
            let query_file = query_file?;
            let query_file_path = query_file.path();
            let query_file_content = std::fs::read_to_string(&query_file_path)?;

            let parsed_query = surrealdb::sql::parse(&query_file_content)?;
            let query_statements = parsed_query.0 .0;

            let _is_multi_statements_query = query_statements.len() > 1;

            let variables = extract_query_variables(&query_file_content)?;

            let query_name = query_file_path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let response_type = format!("{}_Query", query_name).to_case(Case::Pascal);

            let content =
                generate_from_query_template(query_name.to_string(), variables, response_type)?;

            queries_to_generate.insert(query_name, content);
        }

        has_queries_to_generate = !queries_to_generate.is_empty();
        if has_queries_to_generate {
            let db_dir = src_dir.join("db");
            ensures_folder_exists(&db_dir)?;

            let queries_dir = db_dir.join("queries");
            ensures_folder_exists(&queries_dir)?;

            for (query_name, template) in &queries_to_generate {
                let generated_query_file_name = format!("{}.rs", query_name);
                let generated_query_file_path = queries_dir.join(generated_query_file_name);

                std::fs::write(generated_query_file_path, template)?;
            }

            let queries_mod_file_path = db_dir.join("queries.rs");

            let queries_mod_file_content = queries_to_generate
                .keys()
                .map(|table_name| format!("pub mod {};", table_name))
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(queries_mod_file_path, queries_mod_file_content)?;
        }
    }

    // Generate mutations
    let mut mutations_to_generate: HashMap<String, String> = HashMap::new();
    let mut has_mutations_to_generate = false;

    let mutations_dir = Path::new("mutations");
    if mutations_dir.exists() {
        let mutations_files = mutations_dir.read_dir()?;

        for mutation_file in mutations_files {
            let mutation_file = mutation_file?;
            let mutation_file_path = mutation_file.path();
            let mutation_file_content = std::fs::read_to_string(&mutation_file_path)?;

            let parsed_query = surrealdb::sql::parse(&mutation_file_content)?;
            let mutation_statements = parsed_query.0 .0;

            let _is_multi_statements_query = mutation_statements.len() > 1;

            let variables = extract_query_variables(&mutation_file_content)?;

            let mutation_name = mutation_file_path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let response_type = format!("{}_Mutation", mutation_name).to_case(Case::Pascal);

            let content = generate_from_mutation_template(
                mutation_name.to_string(),
                variables,
                response_type,
            )?;

            mutations_to_generate.insert(mutation_name, content);
        }

        has_mutations_to_generate = !mutations_to_generate.is_empty();
        if has_mutations_to_generate {
            let db_dir = src_dir.join("db");
            ensures_folder_exists(&db_dir)?;

            let mutations_dir = db_dir.join("mutations");
            ensures_folder_exists(&mutations_dir)?;

            for (mutation_name, template) in &mutations_to_generate {
                let generated_mutation_file_name = format!("{}.rs", mutation_name);
                let generated_mutation_file_path = mutations_dir.join(generated_mutation_file_name);

                std::fs::write(generated_mutation_file_path, template)?;
            }

            let mutations_mod_file_path = db_dir.join("mutations.rs");

            let mutations_mod_file_content = mutations_to_generate
                .keys()
                .map(|table_name| format!("pub mod {};", table_name))
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(mutations_mod_file_path, mutations_mod_file_content)?;
        }
    }

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

                let struct_fields = extract_struct_fields(define_field_statements, true);

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

            let crud_mod_file_path = db_dir.join("crud.rs");

            let crud_mod_file_content = schemas_to_generate
                .keys()
                .map(|table_name| format!("pub mod {};", table_name))
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(crud_mod_file_path, crud_mod_file_content)?;
        }
    }

    // Generate events
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

                let struct_fields = extract_struct_fields(define_field_statements, false);

                let content = generate_from_event_template(
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

            let events_mod_file_path = db_dir.join("events.rs");

            let events_mod_file_content = events_to_generate
                .keys()
                .map(|table_name| format!("pub mod {};", table_name))
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(events_mod_file_path, events_mod_file_content)?;
        }
    }

    // Generate db.rs
    let has_db_changes = has_queries_to_generate
        || has_mutations_to_generate
        || has_schemas_to_generate
        || has_events_to_generate;
    if has_db_changes {
        let mod_file_path = src_dir.join("db.rs");
        let mut mod_file_modules = vec![];

        if has_schemas_to_generate {
            mod_file_modules.push("crud");
        }
        if has_events_to_generate {
            mod_file_modules.push("events");
        }
        if has_mutations_to_generate {
            mod_file_modules.push("mutations");
        }
        if has_queries_to_generate {
            mod_file_modules.push("queries");
        }

        let mod_file_content = mod_file_modules
            .iter()
            .map(|module| format!("pub mod {};", module))
            .collect::<Vec<_>>()
            .join("\n");

        std::fs::write(mod_file_path, mod_file_content)?;

        println!("db folder generated...");
    } else {
        // TODO : Remove db.rs if it exists
    }

    Ok(())
}

fn extract_struct_fields(
    define_field_statements: Vec<DefineFieldStatement>,
    with_id: bool,
) -> Vec<StructField> {
    let mut struct_fields: HashMap<String, SurrealType> = HashMap::new();

    if with_id {
        struct_fields.insert("id".to_string(), SurrealType::Id);
    }

    for define_field_statement in define_field_statements {
        let field_name = define_field_statement.name.to_string();
        // TODO : Handle other field types
        let field_type = match define_field_statement.kind {
            Some(Kind::String) => SurrealType::String,
            Some(Kind::Record(_)) => SurrealType::Id,
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

fn watch_to_regenerate_db_folder() -> Result<INotifyWatcher> {
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
        println!("Watching schemas folder...");
        watcher.watch(schemas_dir, RecursiveMode::NonRecursive)?;
    }

    let events_dir = Path::new("events");
    if events_dir.exists() {
        println!("Watching events folder...");
        watcher.watch(events_dir, RecursiveMode::NonRecursive)?;
    }

    let queries_dir = Path::new("queries");
    if queries_dir.exists() {
        println!("Watching queries folder...");
        watcher.watch(queries_dir, RecursiveMode::NonRecursive)?;
    }

    let mutations_dir = Path::new("mutations");
    if mutations_dir.exists() {
        println!("Watching mutations folder...");
        watcher.watch(mutations_dir, RecursiveMode::NonRecursive)?;
    }

    Ok(watcher)
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

fn generate_from_event_template(
    func_name: String,
    table_name: String,
    struct_name: String,
    struct_fields: Vec<StructField>,
) -> Result<String> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file("event.rs.jinja2")
        .context("Cannot get template 'event.rs.jinja2'")?
        .contents_utf8()
        .context("Cannot get template 'event.rs.jinja2'")?
        .to_string();

    let content = Environment::new().render_str(
        &template_content,
        context! { func_name, table_name, struct_name, struct_fields },
    )?;

    Ok(content)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct QueryVariables {
    name: String,
    type_: String,
}

fn extract_query_variables(input: &str) -> Result<Vec<QueryVariables>> {
    let variable_regex = Regex::new(r#"^\s*(?:/{2,}|#+)\s*\$(\w+)\s*[:]\s*(\S+)\s*$"#)?;

    let variables = input
        .lines()
        .filter_map(|line| {
            let mut captures = variable_regex.captures_iter(line);

            if let Some(capture) = captures.next() {
                let name = capture[1].to_string();
                let type_ = capture[2].to_string();

                Some(QueryVariables { name, type_ })
            } else {
                None
            }
        })
        .collect();

    Ok(variables)
}

fn generate_from_query_template(
    file_name: String,
    variables: Vec<QueryVariables>,
    response_type: String,
) -> Result<String> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file("query.rs.jinja2")
        .context("Cannot get template 'query.rs.jinja2'")?
        .contents_utf8()
        .context("Cannot get template 'query.rs.jinja2'")?
        .to_string();

    let content = Environment::new().render_str(
        &template_content,
        context! { file_name, variables, response_type },
    )?;

    Ok(content)
}

fn generate_from_mutation_template(
    file_name: String,
    variables: Vec<QueryVariables>,
    response_type: String,
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
        context! { file_name, variables, response_type },
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: Thing,
    pub title: String,
    pub content: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptMigration {
    pub id: Thing,
    pub script_name: String,
    pub executed_at: String,
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
        let struct_fields = vec![StructField {
            name: "post_id".to_string(),
            type_str: "Thing".to_string(),
        }];

        let result = generate_from_event_template(
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishPostData {
    pub post_id: Thing,
}

pub async fn publish_post<C: Connection>(db: &'_ Surreal<C>, data: PublishPostData) -> Result<PublishPostData> {
    let record: PublishPostData = db.create(\"publish_post\").content(data).await?;
    Ok(record)
}"
        );
    }

    #[test]
    fn generate_posts_query_content() {
        let file_name = "posts";
        let variables = vec![];
        let response_type = "PostsQuery";

        let result = generate_from_query_template(
            file_name.to_string(),
            variables,
            response_type.to_string(),
        )
        .unwrap();

        assert_eq!(
            result,
            "use surrealdb::{Surreal, Connection, Result};

use crate::models::queries::PostsQuery;

pub async fn query_posts<C: Connection>(
    db: &'_ Surreal<C>
) -> Result<PostsQuery> {
    const QUERY: &str = include_str!(\"../../../queries/posts.surql\");

    let result: PostsQuery = db
        .query(QUERY)
        .await?
        .take(0)?;

    Ok(result)
}"
        );
    }

    #[test]
    fn generate_post_by_id_query_content() {
        let file_name = "post_by_id";
        let variables = vec![QueryVariables {
            name: "post_id".to_string(),
            type_: "&str".to_string(),
        }];
        let response_type = "PostByIdQuery";

        let result = generate_from_query_template(
            file_name.to_string(),
            variables,
            response_type.to_string(),
        )
        .unwrap();

        assert_eq!(
            result,
            "use surrealdb::{Surreal, Connection, Result};

use crate::models::queries::PostByIdQuery;

pub async fn query_post_by_id<C: Connection>(
    db: &'_ Surreal<C>,
    post_id: &str
) -> Result<PostByIdQuery> {
    const QUERY: &str = include_str!(\"../../../queries/post_by_id.surql\");

    let result: PostByIdQuery = db
        .query(QUERY)
        .bind((\"post_id\", post_id))
        .await?
        .take(0)?;

    Ok(result)
}"
        );
    }

    #[test]
    fn generate_comment_on_post_mutation_content() {
        let file_name = "comment_on_post";
        let variables = vec![
            QueryVariables {
                name: "post_id".to_string(),
                type_: "Option<String>".to_string(),
            },
            QueryVariables {
                name: "content".to_string(),
                type_: "&str".to_string(),
            },
        ];
        let response_type = "CommentOnPostMutation";

        let result = generate_from_mutation_template(
            file_name.to_string(),
            variables,
            response_type.to_string(),
        )
        .unwrap();

        assert_eq!(
            result,
            "use surrealdb::{Surreal, Connection, Result};

use crate::models::mutations::CommentOnPostMutation;

pub async fn mutate_comment_on_post<C: Connection>(
    db: &'_ Surreal<C>,
    post_id: Option<String>,
    content: &str
) -> Result<CommentOnPostMutation> {
    const QUERY: &str = include_str!(\"../../../mutations/comment_on_post.surql\");

    let result: CommentOnPostMutation = db
        .query(QUERY)
        .bind((\"post_id\", post_id))
        .bind((\"content\", content))
        .await?
        .take(0)?;

    Ok(result)
}"
        );
    }

    #[test]
    fn should_extract_no_variable_from_posts_query() {
        const QUERY_CONTENT: &str = include_str!("../templates/projects/blog/queries/posts.surql");

        let variables = extract_query_variables(QUERY_CONTENT).unwrap();

        assert!(variables.is_empty());
    }

    #[test]
    fn should_extract_post_id_variable_from_post_by_id_query() {
        const QUERY_CONTENT: &str =
            include_str!("../templates/projects/blog/queries/post_by_id.surql");

        let variables = extract_query_variables(QUERY_CONTENT).unwrap();

        assert_eq!(
            variables,
            vec![QueryVariables {
                name: "post_id".to_string(),
                type_: "&str".to_string()
            }]
        );
    }
    #[test]
    fn should_extract_multiple_variables_from_comment_mutation() {
        const QUERY_CONTENT: &str =
            include_str!("../templates/projects/blog/mutations/comment.surql");

        let variables = extract_query_variables(QUERY_CONTENT).unwrap();

        assert_eq!(
            variables,
            vec![
                QueryVariables {
                    name: "user_id".to_string(),
                    type_: "&str".to_string()
                },
                QueryVariables {
                    name: "post_id".to_string(),
                    type_: "Option<String>".to_string()
                },
                QueryVariables {
                    name: "comment_id".to_string(),
                    type_: "Option<String>".to_string()
                },
                QueryVariables {
                    name: "content".to_string(),
                    type_: "&str".to_string()
                },
            ]
        );
    }
}
