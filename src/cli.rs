use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "ultime", version, author = "Odonno")]
/// The ultimate full-stack experience
pub struct Args {
    #[command(subcommand)]
    pub command: Action,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Create a new ultime project
    New { name: String },
}
