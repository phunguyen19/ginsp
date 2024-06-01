pub(crate) mod diagnostic;
pub(crate) mod diff_message;
pub(crate) mod update;
pub(crate) mod version;

use clap::Parser;

/// Small utils tools to update local git and compare the commits.
#[derive(Parser, Debug)]
#[clap(name = "ginsp")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Version of the tool.
    #[clap(name = "version", alias = "v")]
    Version,

    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    #[clap(name = "update", alias = "u")]
    Update(Update),

    /// Compare two branches by commit messages.
    #[clap(name = "diff-message", alias = "dm")]
    DiffMessage(DiffMessageParams),

    /// Diagnostic command to check if the tool is working.
    #[clap(name = "diagnostic", alias = "dia")]
    Diagnostic,
}

#[derive(Parser, Debug)]
pub struct Update {
    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    #[clap(name = "branches", required = true)]
    pub branches: Vec<String>,

    #[clap(short, long, default_value = "false")]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct DiffMessageParams {
    /// Two branches to compare.
    #[clap(name = "branches", required = true)]
    pub branches: Vec<String>,

    /// `cherry-pick` commits that contains the given string.
    /// Multiple strings can be separated by comma.
    /// For example: `ginsp diff-message master develop -c "fix,feat"`
    #[clap(short = 'c', long = "cherry-picks", num_args = 1)]
    pub pick_contains: Option<String>,

    /// Fetching ticket status from project management tool
    /// and print it in the result table. This option requires a config file.
    /// For example: `ginsp diff-message master develop -p`
    #[clap(short = 't', long = "ticket-status", default_value = "false")]
    pub is_fetch_ticket_status: bool,

    #[clap(short, long, default_value = "false")]
    pub verbose: bool,
}

impl Cli {
    pub fn run() -> anyhow::Result<()> {
        let options = Cli::parse();

        match &options.subcommand {
            SubCommand::Version => {
                version::Version::new().execute(&options)?;
            }
            SubCommand::Update(_) => {
                update::Update::new().execute(&options)?;
            }
            SubCommand::DiffMessage(_) => {
                diff_message::DiffMessage::new().execute(&options)?;
            }
            SubCommand::Diagnostic => {
                diagnostic::Diagnostic::new().execute(&options)?;
            }
        }

        Ok(())
    }
}

pub trait CommandHandler {
    fn execute(&self, cli: &Cli) -> anyhow::Result<()>;
}
