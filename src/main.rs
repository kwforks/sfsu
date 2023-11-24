#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

// TODO: Replace regex with glob

mod commands;

use clap::Parser;

use commands::Commands;

use sfsu::get_scoop_path;

pub(crate) trait ResultIntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T, E> ResultIntoOption<T> for Result<T, E> {
    fn into_option(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

/// Scoop utilities that can replace the slowest parts of Scoop, and run anywhere from 30-100 times faster
#[derive(Debug, Parser)]
#[clap(about, long_about, author, version)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[clap(long, global = true, help = "Disable terminal formatting")]
    no_color: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.no_color {
        colored::control::set_override(false);
    }

    args.command.run()
}
