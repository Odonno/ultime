![Crates.io](https://img.shields.io/crates/v/ultime) ![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Odonno/ultime/main.yml) ![GitHub](https://img.shields.io/github/license/Odonno/ultime) [![codecov](https://codecov.io/gh/Odonno/ultime/branch/main/graph/badge.svg?token=8DCQY63QP9)](https://codecov.io/gh/Odonno/ultime)

# Ultime

The ultimate full-stack experience.

> **Warning**
> This project is in early preview.

This project can be used as a CLI:

```
cargo install ultime
```

## Get started

This project includes features that will give you a faster and better development workflow using the following tech stack:

- [Leptos](https://leptos.dev/)
- [SurrealDB](https://surrealdb.com/)
- [Rust](https://www.rust-lang.org/)

The main feature is the automatic of code generation based on your SurrealDB schemas and queries. An ultime project should look like this:

- `/schemas` · schemas folder from surrealdb-migrations
- `/events` · events folder from surrealdb-migrations
- `/migrations` · migrations folder from surrealdb-migrations
- `/queries` · a list of .surql files that contains the specific queries for your project
- `/src`
  - `/api` · list of API endpoints
  - `/components` · list of components that can be used anywhere
  - `/db`
    - `/crud` · functions for basic CRUD operations, generated from `/schemas` files
    - `/queries` · functions to query db, generated from `/queries` files
    - `/mutations` · functions to update db, generated from `/events` files
  - `/models` · list of structs used in the app
    - `queries.rs` · types of the response of each query from `/queries` files (this file is currently not automatically generated)
  - `/pages` · list of higher order components that can be used as a route

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
