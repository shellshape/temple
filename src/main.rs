mod commands;
mod template;

use anyhow::Result;
use clap::{command, Parser};
use commands::*;
use env_logger::{fmt::style, Env};
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

// List the names of your sub commands here.
register_commands! {
    Build
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let s = style::AnsiColor::White.on_default().dimmed();

            let style = buf.default_level_style(record.level());
            let timestamp = buf.timestamp();
            let lvl = record.level();
            let args = record.args();

            writeln!(
                buf,
                "{timestamp} {s}[{s:#}{style}{lvl:<5}{style:#}{s}]{s:#} {args}"
            )
        })
        .init();

    cli.commands.run()?;

    Ok(())
}
