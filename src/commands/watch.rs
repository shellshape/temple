use super::Command;
use crate::template::Builder;
use anyhow::Result;
use clap::Args;
use notify::{Event, Watcher};
use std::{path::PathBuf, sync::mpsc};

/// Say hello to world or someone you want to greet
#[derive(Args)]
pub struct Watch {
    #[arg(short, long, default_value = "src")]
    source: PathBuf,

    #[arg(short, long, default_value = "dist")]
    output: PathBuf,
}

impl Command for Watch {
    fn run(&self) -> Result<()> {
        let builder = Builder::new(&self.source, &self.output);

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(&self.source, notify::RecursiveMode::Recursive)?;

        log::info!("Initial build ...");
        builder.build()?;

        log::info!("Watching for changes in {:?} ...", &self.source);

        for res in rx {
            match res {
                Ok(e) => {
                    log::info!("Change detected {:?}: {:?}", e.kind, e.paths);
                    builder.build()?;
                }
                Err(_) => todo!(),
            }
        }

        Ok(())
    }
}
