use anyhow::Result;
use clap::Parser;
use cli::Action;

use crate::cli::Args;

mod cli;
mod new;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Action::New { name } => new::main(name),
    }
}
