use anyhow::{anyhow, Result};
use std::{
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};

use crate::generate;

pub async fn main(open: bool) -> Result<()> {
    if !is_valid_ultime_project() {
        return Err(anyhow!("This is not a valid ultime project"));
    }

    start_surrealdb_instance().await?;
    generate::db::generate_db_folder()?;
    start_leptos_app(open)?;

    // 💡 prevent watcher to be dropped
    let _watcher = generate::db::watch_to_regenerate_db_folder()?;

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

fn start_leptos_app(open: bool) -> Result<()> {
    println!("Start leptos app...");
    let _start_leptos_app = Command::new("cargo").arg("leptos").arg("watch").spawn()?;

    // TODO : Handle error if port (3000) already in use
    // TODO : Use a different port if 3000 is already in use

    if open {
        open_app_in_browser()?;
    }

    Ok(())
}

fn open_app_in_browser() -> Result<()> {
    const APP_URL: &str = "http://localhost:3000";

    println!("Opening app in browser...");
    open::that(APP_URL)?;

    Ok(())
}
