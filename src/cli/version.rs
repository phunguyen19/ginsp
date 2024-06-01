use crate::cli::Cli;
use crate::cli::CommandHandler;
use anyhow::Result;

pub struct Version {}

impl CommandHandler for Version {
    fn execute(&self, _: &Cli) -> Result<()> {
        println!("ginsp version {}", env!("CARGO_PKG_VERSION"));
        anyhow::Ok(())
    }
}

impl Version {
    pub fn new() -> Self {
        Self {}
    }
}
