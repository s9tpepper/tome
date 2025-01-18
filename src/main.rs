use std::env;

use clap::{Parser, Subcommand};

mod app;
mod app_themes;
mod code_gen;
mod compatibility;
mod components;
mod fs;
mod messages;
mod options;
mod projects;
mod requests;
mod templates;
mod theme;
mod themes;

use crate::app::app;

#[derive(Debug, Subcommand)]
enum Cmds {
    /// Does stuff
    Test,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Cmds,
}

#[quit::main]
fn main() -> anyhow::Result<()> {
    let args = env::args().collect::<Vec<_>>();

    match args.len() {
        1 => {
            app()?;

            Ok(())
        }

        _ => {
            // let cli = Cli::parse();

            // match cli.commands {
            //     Cmds::Test => Ok::<(), anyhow::Error>(()),
            // };

            Ok(())
        }
    }
}
