use super::Command;
use crate::server::run_dev_server;
use crate::template::Builder;
use anyhow::Result;
use clap::Args;
use notify::{Event, EventKind, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use tokio::sync::broadcast;

/// Watches the given source directory for changes and rebuilds if detected
#[derive(Args)]
pub struct Watch {
    /// Source directory
    #[arg(short, long, default_value = "src")]
    source: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,

    /// Address to bind dev server to
    #[arg(short, long, default_value = "127.0.0.1:8081")]
    address: String,

    /// Prevent opening the browser with the live server page
    #[arg(long)]
    no_open: bool,
}

impl Command for Watch {
    fn run(&self) -> Result<()> {
        let builder = Builder::new(&self.source, &self.output);

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(&self.source, notify::RecursiveMode::Recursive)?;

        log::info!("Initial build ...");
        builder.build()?;

        let (tx, _) = broadcast::channel(1);

        {
            let url = format!("http://{}", &self.address);
            log::info!("Running internal dev server on {url}");

            let address = self.address.to_owned();
            let output_path = self.output.to_owned();
            let tx = tx.clone();
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
                rt.block_on(run_dev_server(address, output_path, tx))
                    .expect("run dev server");
            });

            if !self.no_open {
                webbrowser::open(&url)?;
            }
        }

        watch_handler(&self.source, rx, &tx, &builder);

        Ok(())
    }
}

fn watch_handler(
    path: &Path,
    rx: Receiver<notify::Result<Event>>,
    tx: &broadcast::Sender<()>,
    builder: &Builder,
) {
    log::info!("Watching for changes in {:?}", path);

    for res in rx {
        match res {
            Ok(e)
                if matches!(
                    e.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) =>
            {
                log::info!("Change detected {:?}: {:?}", e.kind, e.paths);
                match builder.build() {
                    Err(err) => log::error!("build failed: {err}"),
                    Ok(_) => {
                        tx.send(()).expect("send reload message");
                    }
                }
            }
            Err(err) => log::error!("event failed: {err}"),
            _ => {}
        }
    }
}
