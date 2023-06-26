use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[clap(name = "ultime", version, author = "Odonno")]
/// The ultimate full-stack experience
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Action>,
    /// Open browser when app is launched
    #[clap(short, long)]
    pub open: bool,
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
    /// Generate new files through templates
    #[clap(aliases = vec!["g"])]
    Generate {
        #[command(subcommand)]
        command: GenerateAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum GenerateAction {
    /// Generate `db` module inside the `/db` folder
    Db {
        /// Watch file changes to re-generate the `db` module
        #[clap(short, long)]
        watch: bool,
    },
    /// Generate a new leptos component inside the `/components` folder
    #[clap(aliases = vec!["c"])]
    Component {
        /// Name of the component to generate
        name: String,
    },
    /// Generate a new leptos page inside the `/pages` folder
    #[clap(aliases = vec!["p"])]
    Page {
        /// Name of the page to generate
        name: String,
    },
    /// Generate a new leptos endpoint inside the `/api` folder
    Endpoint {
        /// Name of the api endpoint to generate
        name: String,
        /// Use a SurrealDB query from `/queries` to generate the endpoint
        #[clap(long, conflicts_with_all(&["from_mutation", "from_event"]))]
        from_query: Option<String>,
        /// Use a SurrealDB query from `/mutations` to generate the endpoint
        #[clap(long, conflicts_with_all(&["from_query", "from_event"]))]
        from_mutation: Option<String>,
        /// Use a SurrealDB query from `/events` to generate the endpoint
        #[clap(long, conflicts_with_all(&["from_query", "from_mutation"]))]
        from_event: Option<String>,
    },
}
