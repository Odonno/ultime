use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::common::{extract_query_variables, QueryVariable};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryDetailsForEndpoint {
    name: String,
    short_name: String,
    params: Vec<QueryVariable>,
}

pub fn main(name: String, from_query: Option<String>) -> Result<()> {
    let query = get_query_details_for_endpoint(from_query)?;

    let src_dir = Path::new("src");

    let api_dir = src_dir.join("api");
    ensures_folder_exists(&api_dir)?;

    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file("endpoint.rs.jinja2")
        .context("Cannot get template 'endpoint.rs.jinja2'")?
        .contents_utf8()
        .context("Cannot get template 'endpoint.rs.jinja2'")?
        .to_string();

    let endpoint_name = name.to_case(Case::Pascal);
    let function_name = name.to_case(Case::Snake);
    let output_type = match query.clone() {
        Some(query) => format!("{}-query", query.short_name).to_case(Case::Pascal),
        None => "()".to_string(),
    };

    let mut env = Environment::new();

    fn flatten(value: Vec<Vec<String>>) -> Vec<String> {
        value.into_iter().flatten().collect::<Vec<_>>()
    }

    env.add_filter("flatten", flatten);

    let content = env.render_str(
        &template_content,
        context! { endpoint_name, function_name, output_type, query },
    )?;

    let file_name = name.to_case(Case::Snake);
    let file_name = format!("{}.rs", file_name);

    let endpoint_file = api_dir.join(file_name);

    std::fs::write(endpoint_file, content)?;

    println!("Endpoint {} successfully created", name);

    Ok(())
}

fn ensures_folder_exists(dir_path: &PathBuf) -> Result<()> {
    if !dir_path.exists() {
        fs_extra::dir::create_all(dir_path, false)?;
    }

    Ok(())
}

fn get_query_details_for_endpoint(
    from_query: Option<String>,
) -> Result<Option<QueryDetailsForEndpoint>> {
    let result = match from_query {
        Some(from_query) => {
            let queries_dir = Path::new("queries");

            let query_name = get_query_name(from_query);

            let query_file_name = format!("{}.surql", query_name);
            let query_file = queries_dir.join(&query_file_name);

            if !query_file.exists() {
                return Err(anyhow!(format!(
                    "Query '{}' does not exist",
                    query_file_name
                )));
            }

            let query_content = std::fs::read_to_string(&query_file)?;

            let params = extract_query_variables(&query_content)?;

            let details = QueryDetailsForEndpoint {
                name: format!("query-{}", query_name).to_case(Case::Snake),
                short_name: query_name.to_case(Case::Snake),
                params,
            };

            Some(details)
        }
        None => None,
    };

    Ok(result)
}

fn get_query_name(from_query: String) -> String {
    let suffixes = [".rs", ".surql"];

    for suffix in suffixes.iter() {
        if from_query.ends_with(suffix) {
            return from_query.replace(suffix, "");
        }
    }

    from_query
}
