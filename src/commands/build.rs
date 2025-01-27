use super::Command;
use crate::template::Builder;
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Builds the static site from the given source
#[derive(Args)]
pub struct Build {
    /// Source directory
    #[arg(short, long, default_value = "src")]
    source: PathBuf,

    /// Output directory
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
