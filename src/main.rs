mod commands;
mod server;
mod template;

use anyhow::Result;
use clap::{command, Parser};
use commands::*;
use env_logger::fmt::style;
use env_logger::Env;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,

    #[arg(short, long, default_value = "info")]
    log_filter: String,
}

// List the names of your sub commands here.
register_commands! {
    Build
    Watch
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_filter))
        .format(|buf, record| {
            let s_bracket = style::AnsiColor::White.on_default().dimmed();
            let s_module = style::AnsiColor::Magenta.on_default();

            let style = buf.default_level_style(record.level());
            let timestamp = buf.timestamp();
            let lvl = record.level();
            let args = record.args();

            let module = record.module_path().unwrap();

            writeln!(
                buf,
                "{timestamp} {s_bracket}[{s_bracket:#}{style}{lvl:<5}{style:#}{s_bracket}]{s_bracket:#} {s_module}{module}{s_module:#} {args}"
            )
        })
        .init();

    cli.commands.run()?;

    Ok(())
}
