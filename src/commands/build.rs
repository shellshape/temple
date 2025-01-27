use std::path::PathBuf;

use crate::template::Builder;

use super::Command;
use anyhow::Result;
use clap::Args;

/// Say hello to world or someone you want to greet
#[derive(Args)]
pub struct Build {
    #[arg(short, long, default_value = "src")]
    source: PathBuf,

    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}

impl Command for Build {
    fn run(&self) -> Result<()> {
        let builder = Builder::new(&self.source, &self.output);
        builder.build()?;
        Ok(())
    }
}
