use anyhow::Result;
use clap::Parser;
use cli::{Action, GenerateAction};
use generate::endpoint::GenerateEndpointArgs;

use crate::cli::Args;

mod cli;
mod generate;
mod new;
mod run;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        None => run::main(args.open).await,
        Some(command) => match command {
            Action::New { name, template } => new::main(name, template),
            Action::Generate { command } => match command {
                GenerateAction::Db { watch } => generate::db::main(watch),
                GenerateAction::Component { name } => generate::component::main(name),
                GenerateAction::Page { name } => generate::page::main(name),
                GenerateAction::Endpoint {
                    name,
                    from_query,
                    from_mutation,
                } => {
                    let args = GenerateEndpointArgs {
                        name,
                        from_query,
                        from_mutation,
                    };
                    generate::endpoint::main(args)
                }
            },
        },
    }
}
