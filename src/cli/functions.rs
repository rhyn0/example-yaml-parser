use crate::{Lexer, Parser};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FunctionError {
    #[error("Invalid YAML file `{0}`")]
    InvalidYaml(PathBuf),
}

pub type CliFunctionResult = Result<(), FunctionError>;

pub fn lex_file(file: std::path::PathBuf) -> CliFunctionResult {
    let reader = BufReader::new(File::open(file).unwrap());
    let mut lexer = Lexer::new(reader.bytes().map(|b| char::from(b.unwrap())));

    for x in lexer.into_iter() {
        eprintln!("{x:?}");
    }
    Ok(())
}
