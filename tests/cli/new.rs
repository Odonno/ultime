use anyhow::Result;
use assert_fs::{fixture::PathChild, prelude::PathAssert};
use chrono::Local;

use crate::helpers::*;

#[test]
fn create_new_project_with_empty_template() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?.into_persistent();

    let mut cmd = create_cmd()?;
    cmd.current_dir(&temp_dir).arg("new").arg("ultime-project");

    cmd.assert().success().stdout(
        "Cloning template...
Creating migration project...
Project 'ultime-project' created. Run the following commands:
cd ultime-project
ultime\n",
    );

    let my_blog_folder = temp_dir.child("ultime-project");
    assert!(my_blog_folder.is_dir(), "ultime-project dir should exists");

    {
        let cargo_toml_jinja2_file = my_blog_folder.child("Cargo.toml.jinja2");
        assert!(
            !cargo_toml_jinja2_file.exists(),
            "Cargo.toml.jinja2 file should not exists"
        );

        let cargo_toml_file = my_blog_folder.child("Cargo.toml");
        assert!(cargo_toml_file.is_file(), "Cargo.toml file should exists");
        cargo_toml_file.assert(
            r#"[package]
name = "ultime-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
actix-files = { version = "0.6", optional = true }
actix-web = { version = "4", optional = true, features = ["macros"] }
console_error_panic_hook = "0.1"
cfg-if = "1"
leptos = { version = "0.3", default-features = false, features = [
  "serde",
] }
leptos_meta = { version = "0.3", default-features = false }
leptos_actix = { version = "0.3", optional = true }
leptos_router = { version = "0.3", default-features = false }
serde = { version = "1.0.163", features = ["derive"] }
surrealdb = { version = "1.0.0-beta.9" }
wasm-bindgen = "=0.2.86"

[features]
csr = ["leptos/csr", "leptos_meta/csr", "leptos_router/csr"]
hydrate = ["leptos/hydrate", "leptos_meta/hydrate", "leptos_router/hydrate"]
ssr = [
  "dep:actix-files",
  "dep:actix-web",
  "dep:leptos_actix",
  "leptos/ssr",
  "leptos_meta/ssr",
  "leptos_router/ssr",
]

[package.metadata.leptos]
# The name used by wasm-bindgen/cargo-leptos for the JS/WASM bundle. Defaults to the crate name   
output-name = "ultime-project"
# The site root folder is where cargo-leptos generate all output. WARNING: all content of this folder will be erased on a rebuild. Use it in your server setup.
site-root = "target/site"
# The site-root relative folder where all compiled output (JS, WASM and CSS) is written
# Defaults to pkg	
site-pkg-dir = "pkg"
# [Optional] The source CSS file. If it ends with .sass or .scss then it will be compiled by dart-sass into CSS. The CSS is optimized by Lightning CSS before being written to <site-root>/<site-pkg>/app.css
style-file = "style/main.scss"
# Assets source dir. All files found here will be copied and synchronized to site-root.
# The assets-dir cannot have a sub directory with the same name/path as site-pkg-dir.
#
# Optional. Env: LEPTOS_ASSETS_DIR.
assets-dir = "assets"
# The IP and port (ex: 127.0.0.1:3000) where the server serves the content. Use it in your server setup.
site-addr = "127.0.0.1:3000"
# The port to use for automatic reload monitoring
reload-port = 3001
# [Optional] Command to use when running end2end tests. It will run in the end2end dir.
#   [Windows] for non-WSL use "npx.cmd playwright test"
#   This binary name can be checked in Powershell with Get-Command npx
end2end-cmd = "npx playwright test"
end2end-dir = "end2end"
#  The browserlist query used for optimizing the CSS.
browserquery = "defaults"
# Set by cargo-leptos watch when building with that tool. Controls whether autoreload JS will be included in the head
watch = false
# The environment Leptos will run in, usually either "DEV" or "PROD"
env = "DEV"
# The features to use when compiling the bin target
#
# Optional. Can be over-ridden with the command line parameter --bin-features
bin-features = ["ssr"]

# If the --no-default-features flag should be used when compiling the bin target
#
# Optional. Defaults to false.
bin-default-features = false

# The features to use when compiling the lib target
#
# Optional. Can be over-ridden with the command line parameter --lib-features
lib-features = ["hydrate"]

# If the --no-default-features flag should be used when compiling the lib target
#
# Optional. Defaults to false.
lib-default-features = false"#,
        );
    }

    {
        let index_html_jinja2_file = my_blog_folder.child("index.html.jinja2");
        assert!(
            !index_html_jinja2_file.exists(),
            "index.html.jinja2 file should not exists"
        );

        let index_html_file = my_blog_folder.child("index.html");
        assert!(index_html_file.is_file(), "index.html file should exists");
        index_html_file.assert(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>ultime-project</title>
    <link data-trunk rel="rust" data-wasm-opt="0" data-keep-debug="true" />
  </head>
  <body></body>
</html>"#,
        );
    }

    {
        let gitignore_file = my_blog_folder.child(".gitignore");
        assert!(gitignore_file.is_file(), ".gitignore file should exists");
        gitignore_file.assert(
            r#"# Generated by Cargo
# will have compiled files and executables
/target/
pkg

# These are backup files generated by rustfmt
**/*.rs.bk

# node e2e test tools and outputs
node_modules/
test-results/
end2end/playwright-report/
playwright/.cache/
"#,
        );
    }

    {
        let schemas_folder = my_blog_folder.child("schemas");
        assert!(schemas_folder.is_dir(), "schemas dir should exist");
    }

    {
        let events_folder = my_blog_folder.child("events");
        assert!(!events_folder.exists(), "events dir should not exist");
    }

    {
        let migrations_folder = my_blog_folder.child("migrations");
        assert!(
            !migrations_folder.exists(),
            "migrations dir should not exist"
        );
    }

    let src_folder = my_blog_folder.child("src");

    {
        let app_rs_jinja2_file = my_blog_folder.child("app.rs.jinja2");
        assert!(
            !app_rs_jinja2_file.exists(),
            "app.rs.jinja2 file should not exists"
        );

        let app_rs_file = src_folder.child("app.rs");
        assert!(app_rs_file.is_file(), "app.rs file should exists");
        app_rs_file.assert(
            r#"use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::home::HomePage;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/ultime-project.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}"#,
        );
    }

    {
        let main_rs_jinja2_file = my_blog_folder.child("main.rs.jinja2");
        assert!(
            !main_rs_jinja2_file.exists(),
            "main.rs.jinja2 file should not exists"
        );

        let main_rs_file = src_folder.child("main.rs");
        assert!(main_rs_file.is_file(), "main.rs file should exists");
        main_rs_file.assert(
            r#"#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use ultime_project::app::*;

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `ssg` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features ssg`
    use leptos::*;
    use ultime_project::app::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| {
        // note: for testing it may be preferrable to replace this with a
        // more specific component, although leptos_router should still work
        view! {cx, <App/> }
    });
}"#,
        );
    }

    temp_dir.close()?;

    Ok(())
}

#[test]
fn create_new_project_with_blog_template() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?.into_persistent();

    let now = Local::now();

    let mut cmd = create_cmd()?;
    cmd.current_dir(&temp_dir)
        .arg("new")
        .arg("my-blog")
        .arg("--template")
        .arg("blog");

    cmd.assert().success().stdout(
        "Cloning template...
Creating migration project...
Project 'my-blog' created. Run the following commands:
cd my-blog
ultime\n",
    );

    let my_blog_folder = temp_dir.child("my-blog");
    assert!(my_blog_folder.is_dir(), "my-blog dir should exists");

    {
        let cargo_toml_jinja2_file = my_blog_folder.child("Cargo.toml.jinja2");
        assert!(
            !cargo_toml_jinja2_file.exists(),
            "Cargo.toml.jinja2 file should not exists"
        );

        let cargo_toml_file = my_blog_folder.child("Cargo.toml");
        assert!(cargo_toml_file.is_file(), "Cargo.toml file should exists");
        cargo_toml_file.assert(
            r#"[package]
name = "my-blog"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
actix-files = { version = "0.6", optional = true }
actix-web = { version = "4", optional = true, features = ["macros"] }
console_error_panic_hook = "0.1"
cfg-if = "1"
leptos = { version = "0.3", default-features = false, features = [
  "serde",
] }
leptos_meta = { version = "0.3", default-features = false }
leptos_actix = { version = "0.3", optional = true }
leptos_router = { version = "0.3", default-features = false }
serde = { version = "1.0.163", features = ["derive"] }
surrealdb = { version = "1.0.0-beta.9" }
wasm-bindgen = "=0.2.86"

[features]
csr = ["leptos/csr", "leptos_meta/csr", "leptos_router/csr"]
hydrate = ["leptos/hydrate", "leptos_meta/hydrate", "leptos_router/hydrate"]
ssr = [
  "dep:actix-files",
  "dep:actix-web",
  "dep:leptos_actix",
  "leptos/ssr",
  "leptos_meta/ssr",
  "leptos_router/ssr",
]

[package.metadata.leptos]
# The name used by wasm-bindgen/cargo-leptos for the JS/WASM bundle. Defaults to the crate name   
output-name = "my-blog"
# The site root folder is where cargo-leptos generate all output. WARNING: all content of this folder will be erased on a rebuild. Use it in your server setup.
site-root = "target/site"
# The site-root relative folder where all compiled output (JS, WASM and CSS) is written
# Defaults to pkg	
site-pkg-dir = "pkg"
# [Optional] The source CSS file. If it ends with .sass or .scss then it will be compiled by dart-sass into CSS. The CSS is optimized by Lightning CSS before being written to <site-root>/<site-pkg>/app.css
style-file = "style/main.scss"
# Assets source dir. All files found here will be copied and synchronized to site-root.
# The assets-dir cannot have a sub directory with the same name/path as site-pkg-dir.
#
# Optional. Env: LEPTOS_ASSETS_DIR.
assets-dir = "assets"
# The IP and port (ex: 127.0.0.1:3000) where the server serves the content. Use it in your server setup.
site-addr = "127.0.0.1:3000"
# The port to use for automatic reload monitoring
reload-port = 3001
# [Optional] Command to use when running end2end tests. It will run in the end2end dir.
#   [Windows] for non-WSL use "npx.cmd playwright test"
#   This binary name can be checked in Powershell with Get-Command npx
end2end-cmd = "npx playwright test"
end2end-dir = "end2end"
#  The browserlist query used for optimizing the CSS.
browserquery = "defaults"
# Set by cargo-leptos watch when building with that tool. Controls whether autoreload JS will be included in the head
watch = false
# The environment Leptos will run in, usually either "DEV" or "PROD"
env = "DEV"
# The features to use when compiling the bin target
#
# Optional. Can be over-ridden with the command line parameter --bin-features
bin-features = ["ssr"]

# If the --no-default-features flag should be used when compiling the bin target
#
# Optional. Defaults to false.
bin-default-features = false

# The features to use when compiling the lib target
#
# Optional. Can be over-ridden with the command line parameter --lib-features
lib-features = ["hydrate"]

# If the --no-default-features flag should be used when compiling the lib target
#
# Optional. Defaults to false.
lib-default-features = false"#,
        );
    }

    {
        let index_html_jinja2_file = my_blog_folder.child("index.html.jinja2");
        assert!(
            !index_html_jinja2_file.exists(),
            "index.html.jinja2 file should not exists"
        );

        let index_html_file = my_blog_folder.child("index.html");
        assert!(index_html_file.is_file(), "index.html file should exists");
        index_html_file.assert(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>my-blog</title>
    <link data-trunk rel="rust" data-wasm-opt="0" data-keep-debug="true" />
  </head>
  <body></body>
</html>"#,
        );
    }

    {
        let gitignore_file = my_blog_folder.child(".gitignore");
        assert!(gitignore_file.is_file(), ".gitignore file should exists");
        gitignore_file.assert(
            r#"# Generated by Cargo
# will have compiled files and executables
/target/
pkg

# These are backup files generated by rustfmt
**/*.rs.bk

# node e2e test tools and outputs
node_modules/
test-results/
end2end/playwright-report/
playwright/.cache/
"#,
        );
    }

    {
        let schemas_folder = my_blog_folder.child("schemas");
        assert!(schemas_folder.is_dir(), "schemas dir should exists");
    }

    {
        let events_folder = my_blog_folder.child("events");
        assert!(events_folder.is_dir(), "events dir should exists");
    }

    {
        let migrations_folder = my_blog_folder.child("migrations");
        assert!(migrations_folder.is_dir(), "migrations dir should exists");

        let add_admin_user_migration_full_name =
            format!("{}01_AddAdminUser.surql", now.format("%Y%m%d_%H%M"));
        let add_admin_user_migration_file =
            migrations_folder.child(add_admin_user_migration_full_name);
        assert!(
            add_admin_user_migration_file.is_file(),
            "AddAdminUser migration file should exists"
        );

        let add_post_migration_full_name = format!("{}02_AddPost.surql", now.format("%Y%m%d_%H%M"));
        let add_post_migration_file = migrations_folder.child(add_post_migration_full_name);
        assert!(
            add_post_migration_file.is_file(),
            "AddPost migration file should exists"
        );

        let comment_post_migration_full_name =
            format!("{}03_CommentPost.surql", now.format("%Y%m%d_%H%M"));
        let comment_post_migration_file = migrations_folder.child(comment_post_migration_full_name);
        assert!(
            comment_post_migration_file.is_file(),
            "CommentPost migration file should exists"
        );
    }

    let src_folder = my_blog_folder.child("src");

    {
        let app_rs_jinja2_file = my_blog_folder.child("app.rs.jinja2");
        assert!(
            !app_rs_jinja2_file.exists(),
            "app.rs.jinja2 file should not exists"
        );

        let app_rs_file = src_folder.child("app.rs");
        assert!(app_rs_file.is_file(), "app.rs file should exists");
        app_rs_file.assert(
            r#"use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::home::HomePage;
use crate::pages::post::PostDetailsPage;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/my-blog.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    <Route path="/posts/:id" view=|cx| view! { cx, <PostDetailsPage/> }/>
                </Routes>
            </main>
        </Router>
    }
}"#,
        );
    }

    {
        let main_rs_jinja2_file = my_blog_folder.child("main.rs.jinja2");
        assert!(
            !main_rs_jinja2_file.exists(),
            "main.rs.jinja2 file should not exists"
        );

        let main_rs_file = src_folder.child("main.rs");
        assert!(main_rs_file.is_file(), "main.rs file should exists");
        main_rs_file.assert(
            r#"#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use my_blog::app::*;
    use my_blog::api::{FetchBlogPosts, FetchPostDetails};

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    let _ = FetchBlogPosts::register();
    let _ = FetchPostDetails::register();

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .service(Files::new("/", site_root))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `ssg` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features ssg`
    use leptos::*;
    use my_blog::app::*;
    use wasm_bindgen::prelude::wasm_bindgen;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| {
        // note: for testing it may be preferrable to replace this with a
        // more specific component, although leptos_router should still work
        view! {cx, <App/> }
    });
}"#,
        );
    }

    temp_dir.close()?;

    Ok(())
}

#[test]
fn fails_to_create_new_project_if_folder_already_exist() -> Result<()> {
    let temp_dir = assert_fs::TempDir::new()?;

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir).arg("new").arg("my-blog");

        cmd.assert().success();
    }

    {
        let mut cmd = create_cmd()?;
        cmd.current_dir(&temp_dir).arg("new").arg("my-blog");

        cmd.assert()
            .failure()
            .stderr("Error: Project already exists\n");
    }

    temp_dir.close()?;

    Ok(())
}
