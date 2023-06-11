use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[clap(name = "ultime", version, author = "Odonno")]
/// The ultimate full-stack experience
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Action>,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum UltimeProjectTemplate {
    Empty,
    Blog,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Create a new ultime project
    New {
        /// Name of the project that will be generated
        name: String,
        /// Template to use
        #[clap(long)]
        template: Option<UltimeProjectTemplate>,
    },
}
