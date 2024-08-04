use std::path::PathBuf;

use clap::{Parser, Subcommand};
use thiserror::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("File `{0}` does not exist.")]
    FileNotFoundError(PathBuf),
}

impl Cli {
    pub fn get_debug(&self) -> u8 {
        self.debug
    }
    pub fn get_file_path(&self) -> Result<PathBuf, CliError> {
        let Commands::Lex { file } = &self.command;
        if !file.exists() || !file.is_file() {
            Err(CliError::FileNotFoundError(file.clone()))
        } else {
            Ok(file.clone())
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    // output the lexed file
    Lex {
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
