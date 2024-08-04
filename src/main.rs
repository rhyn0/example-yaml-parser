mod cli;
mod lexer;
mod parser;
use cli::prelude::*;
use lexer::prelude::*;
use parser::prelude::*;
use std::process::exit;
fn main() {
    let cli = Cli::parse();
    match cli.get_debug() {
        0 => eprint!(""),
        1 => eprintln!("Some debug info is displayed"),
        2 => eprintln!("All debug info is displayed"),
        _ => eprintln!("Are you in need of this much information?"),
    };
    match &cli.command {
        Commands::Lex { file } => {
            let file_path = match cli.get_file_path() {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("{e}");
                    exit(1);
                }
            };
            println!("Lexing a file {:?}", file_path);
            match lex_file(file_path) {
                Ok(_) => exit(0),
                Err(e) => {
                    eprintln!("{e}");
                    exit(1);
                }
            }
        }
    };
}
