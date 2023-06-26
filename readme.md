![Crates.io](https://img.shields.io/crates/v/ultime) ![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Odonno/ultime/main.yml) ![GitHub](https://img.shields.io/github/license/Odonno/ultime) [![codecov](https://codecov.io/gh/Odonno/ultime/branch/main/graph/badge.svg?token=8DCQY63QP9)](https://codecov.io/gh/Odonno/ultime)

# Ultime

The ultimate full-stack experience.

> **Warning**
> This project is in early preview.

This project can be used as a CLI:

```
cargo install ultime
```

## Architecture

This project includes features that will give you a faster and better development workflow using the following tech stack:

- [Leptos](https://leptos.dev/)
- [SurrealDB](https://surrealdb.com/)
- [Rust](https://www.rust-lang.org/)
- [SurrealDB migrations](https://github.com/Odonno/surrealdb-migrations)

The main feature is the automatic of code generation based on your SurrealDB schemas and queries. An ultime project should look like this:

- `/schemas` · schemas folder from surrealdb-migrations
- `/events` · events folder from surrealdb-migrations
- `/migrations` · migrations folder from surrealdb-migrations
- `/queries` · a list of .surql files that contains the specific queries for your project (only fetch data)
- `/mutations` · a list of .surql files that contains the specific mutations for your project (change data)
- `/src`
  - `/api` · list of API endpoints
  - `/components` · list of components that can be used anywhere
  - `/db`
    - `/crud` · functions for basic CRUD operations, generated from `/schemas` files
    - `/events` · functions to execute SurrealDB events, generated from `/events` files
    - `/mutations` · functions to update db, generated from `/mutations` files
    - `/queries` · functions to query db, generated from `/queries` files
  - `/models` · list of structs used in the app
    - `queries.rs` · types of the response of each query from `/queries` files (this file is currently not automatically generated)
    - `mutations.rs` · types of the response of each mutation from `/mutations` files (this file is currently not automatically generated)
  - `/pages` · list of higher order components that can be used as a route

## Get started

First, you need to create a new project using [one of the predefined templates](#predefined-templates). So, for example:

```
ultime new my-blog --template blog
```

A new directory will be created. You can `cd` to this new directory and run the following command:

```
ultime
```

This command will:

- start a new SurrealDB local instance
- apply schemas and migrations automatically
- generate the `db` module from `/schemas`, `/events`, `/queries` and `/mutations` folders
- launch the leptos app

### Automatic code generation of models

As of now, it is not possible to automatically detect the output of a .surql file: `queries` or `mutations`. However, a type is automatically generated for you so that all you need is to define the properties of this type. All models should be defined in the `/src/models` folder.

Here is an example of the model definition from the response of `/queries/posts.surql` query:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostsQueryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub number_of_comments: u16,
}

pub type PostsQuery = Vec<PostsQueryItem>;
```

Another example from the response of `/mutations/comment.surql` query:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentMutationItem {
    pub id: String,
    pub content: String,
}

pub type CommentMutation = Vec<CommentMutationItem>;
```

### Query/Mutation variables extraction

In order to differentiate internal variables and input variables, we established a pattern to follow in order to successfully extract query and mutation variables.

The following mutation...

```sql
// $user_id: String
// $post_id: Option<String>
// $comment_id: Option<String>
// $content: String

LET $user = (SELECT * FROM type::thing("user", $user_id));

LET $post_or_comment = (SELECT * FROM type::thing("post", $post_id), type::thing("comment", $comment_id));

RELATE $user->comment->$post_or_comment
SET content = $content;
```

...will generate this function:

```rust
pub async fn mutate_comment<C: Connection>(
    db: &'_ Surreal<C>,
    user_id: String,
    post_id: Option<String>,
    comment_id: Option<String>,
    content: String
) -> Result<CommentMutation> {
    const QUERY: &str = include_str!("../../../mutations/comment.surql");

    let result: CommentMutation =  db
        .query(QUERY)
        .bind(("user_id", user_id))
        .bind(("post_id", post_id))
        .bind(("comment_id", comment_id))
        .bind(("content", content))
        .await?
        .take(0)?;

    Ok(result)
}
```

As you can see, comments should follow the rules:

- only 1 variable per line
- it should start with the variable name (do not forget the `$` prefix)
- then followed by a colon `:`
- then it should end with the variable type
- whitespaces are allowed according to your coding convention

## Predefined templates

To help you get started quickly, there is a list of predefined templates you can use:

| Template                          | Description                                                                                                                                                                                         |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [empty](templates/projects/empty) | The smallest ultime project you can create.<br /> A clean schema with an already defined `script_migration` table to store the applied migrations.<br /> A basic leptos app with a Counter example. |
| [blog](templates/projects/blog)   | A blog app: create new blog posts, publish/unpublish posts and comments.                                                                                                                            |

You can create a new ultime project using the following command line:

```
ultime new <PROJECT_NAME> --template <TEMPLATE>
```
