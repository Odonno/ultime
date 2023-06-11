use anyhow::Result;
use clap::Parser;
use cli::Action;

use crate::cli::Args;

mod cli;
mod new;
mod run;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        None => run::main(),
        Some(command) => match command {
            Action::New { name, template } => new::main(name, template),
        },
    }
}
