use anyhow::{anyhow, Context, Result};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::common::{extract_query_variables, QueryVariable};

pub struct GenerateEndpointArgs {
    pub name: String,
    pub from_query: Option<String>,
    pub from_mutation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryDetailsForEndpoint {
    name: String,
    short_name: String,
    output_type: String,
    params: Vec<QueryVariable>,
}

enum EndpointTypeGenerated {
    Empty,
    Query(QueryDetailsForEndpoint),
    Mutation(QueryDetailsForEndpoint),
}

pub fn main(args: GenerateEndpointArgs) -> Result<()> {
    let GenerateEndpointArgs {
        name,
        from_query,
        from_mutation,
    } = args;

    let query = get_query_details_for_endpoint(from_query)?;
    let mutation = get_mutation_details_for_endpoint(from_mutation)?;

    let endpoint_type_generated = match (query.clone(), mutation.clone()) {
        (None, None) => EndpointTypeGenerated::Empty,
        (Some(query), None) => EndpointTypeGenerated::Query(query),
        (None, Some(mutation)) => EndpointTypeGenerated::Mutation(mutation),
        _ => return Err(anyhow!("Failed to generate endpoint")),
    };

    let src_dir = Path::new("src");

    let api_dir = src_dir.join("api");
    ensures_folder_exists(&api_dir)?;

    let template_name = get_template_name(endpoint_type_generated);

    let template_content = get_template(template_name)?;

    let endpoint_name = name.to_case(Case::Pascal);
    let function_name = name.to_case(Case::Snake);

    let mut env = Environment::new();

    fn flatten(value: Vec<Vec<String>>) -> Vec<String> {
        value.into_iter().flatten().collect::<Vec<_>>()
    }

    env.add_filter("flatten", flatten);

    let content = env.render_str(
        &template_content,
        context! { endpoint_name, function_name, query, mutation },
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

            let output_type = format!("{}-query", query_name).to_case(Case::Pascal);

            let query_content = std::fs::read_to_string(&query_file)?;

            let params = extract_query_variables(&query_content)?;

            let details = QueryDetailsForEndpoint {
                name: format!("query-{}", query_name).to_case(Case::Snake),
                short_name: query_name.to_case(Case::Snake),
                output_type,
                params,
            };

            Some(details)
        }
        None => None,
    };

    Ok(result)
}

fn get_mutation_details_for_endpoint(
    from_mutation: Option<String>,
) -> Result<Option<QueryDetailsForEndpoint>> {
    let result = match from_mutation {
        Some(from_query) => {
            let mutations_dir = Path::new("mutations");

            let mutation_name = get_query_name(from_query);

            let mutation_file_name = format!("{}.surql", mutation_name);
            let mutation_file = mutations_dir.join(&mutation_file_name);

            if !mutation_file.exists() {
                return Err(anyhow!(format!(
                    "Mutation '{}' does not exist",
                    mutation_file_name
                )));
            }

            let output_type = format!("{}-mutation", mutation_name).to_case(Case::Pascal);

            let mutation_content = std::fs::read_to_string(&mutation_file)?;

            let params = extract_query_variables(&mutation_content)?;

            let details = QueryDetailsForEndpoint {
                name: format!("mutate-{}", mutation_name).to_case(Case::Snake),
                short_name: mutation_name.to_case(Case::Snake),
                output_type,
                params,
            };

            Some(details)
        }
        None => None,
    };

    Ok(result)
}

fn get_query_name(from: String) -> String {
    let suffixes = [".rs", ".surql"];

    for suffix in suffixes.iter() {
        if from.ends_with(suffix) {
            return from.replace(suffix, "");
        }
    }

    from
}

fn get_template_name(endpoint_type_generated: EndpointTypeGenerated) -> String {
    let sub_template_name = match endpoint_type_generated {
        EndpointTypeGenerated::Empty => "empty",
        EndpointTypeGenerated::Query(_) => "query",
        EndpointTypeGenerated::Mutation(_) => "mutation",
    };

    format!("endpoint.{}.rs.jinja2", sub_template_name)
}

fn get_template(template_name: String) -> Result<String> {
    const TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/generate");

    let template_content = TEMPLATES_DIR
        .get_file(&template_name)
        .context(format!("Cannot get template '{}'", template_name))?
        .contents_utf8()
        .context(format!("Cannot get template '{}'", template_name))?
        .to_string();

    Ok(template_content)
}
