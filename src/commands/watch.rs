use super::Command;
use crate::template::Builder;
use anyhow::Result;
use clap::Args;
use notify::{Event, EventKind, Watcher};
use std::{path::PathBuf, sync::mpsc};

/// Watches the given source directory for changes and rebuilds if detected
#[derive(Args)]
pub struct Watch {
    /// Source directory
    #[arg(short, long, default_value = "src")]
    source: PathBuf,

    /// Output directory
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
                Ok(e)
                    if matches!(
                        e.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) =>
                {
                    log::info!("Change detected {:?}: {:?}", e.kind, e.paths);
                    builder.build()?;
                }
                Err(_) => todo!(),
                _ => {}
            }
        }

        Ok(())
    }
}
