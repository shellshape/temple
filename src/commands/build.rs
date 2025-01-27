use super::Command;
use crate::template::Builder;
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

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

        log::info!(
            "Building from {:?} into {:?} ...",
            &self.source,
            &self.output
        );

        builder.build()?;

        log::info!("Build finished successfully! 🎉");

        Ok(())
    }
}
