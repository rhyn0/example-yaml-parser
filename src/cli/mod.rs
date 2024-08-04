mod commands;
mod functions;
pub use commands::{Cli, Commands};
pub use functions::{lex_file, FunctionError};

pub mod prelude {
    pub use super::{lex_file, Cli, Commands, FunctionError};
    pub use clap::Parser;
}
