use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryVariable {
    pub name: String,
    pub type_: String,
}

pub fn extract_query_variables(input: &str) -> Result<Vec<QueryVariable>> {
    let variable_regex = Regex::new(r#"^\s*(?:/{2,}|#+)\s*\$(\w+)\s*[:]\s*(\S+)\s*$"#)?;

    let variables = input
        .lines()
        .filter_map(|line| {
            let mut captures = variable_regex.captures_iter(line);

            if let Some(capture) = captures.next() {
                let name = capture[1].to_string();
                let type_ = capture[2].to_string();

                Some(QueryVariable { name, type_ })
            } else {
                None
            }
        })
        .collect();

    Ok(variables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_extract_no_variable_from_posts_query() {
        const QUERY_CONTENT: &str =
            include_str!("../../templates/projects/blog/queries/posts.surql");

        let variables = extract_query_variables(QUERY_CONTENT).unwrap();

        assert!(variables.is_empty());
    }

    #[test]
    fn should_extract_post_id_variable_from_post_by_id_query() {
        const QUERY_CONTENT: &str =
            include_str!("../../templates/projects/blog/queries/post_by_id.surql");

        let variables = extract_query_variables(QUERY_CONTENT).unwrap();

        assert_eq!(
            variables,
            vec![QueryVariable {
                name: "post_id".to_string(),
                type_: "String".to_string()
            }]
        );
    }

    #[test]
    fn should_extract_multiple_variables_from_comment_mutation() {
        const QUERY_CONTENT: &str =
            include_str!("../../templates/projects/blog/mutations/comment.surql");

        let variables = extract_query_variables(QUERY_CONTENT).unwrap();

        assert_eq!(
            variables,
            vec![
                QueryVariable {
                    name: "user_id".to_string(),
                    type_: "String".to_string()
                },
                QueryVariable {
                    name: "post_id".to_string(),
                    type_: "Option<String>".to_string()
                },
                QueryVariable {
                    name: "comment_id".to_string(),
                    type_: "Option<String>".to_string()
                },
                QueryVariable {
                    name: "content".to_string(),
                    type_: "String".to_string()
                },
            ]
        );
    }
}
