use anyhow::Result;
use clap::Parser;
use cli::Action;

use crate::cli::Args;

mod cli;
mod new;
mod run;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        None => run::main(args.open).await,
        Some(command) => match command {
            Action::New { name, template } => new::main(name, template),
        },
    }
}
