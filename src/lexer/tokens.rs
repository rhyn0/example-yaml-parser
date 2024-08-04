use std::fmt;

use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Debug, Eq)]
pub struct CharacterPosition {
    index: usize,
    pub line: usize,
    pub column: usize,
}

impl CharacterPosition {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn next_index(&mut self) {
        self.index += 1;
    }
}

impl Default for CharacterPosition {
    fn default() -> Self {
        Self {
            index: 0,
            line: 1,
            column: 0,
        }
    }
}

impl fmt::Display for CharacterPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Line: {}, Column: {}", self.line, self.column)
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum YamlTokenType {
    NoToken,
    Scalar(String),
    Key,
    Value,
    Start,
    End,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct YamlToken(pub CharacterPosition, pub YamlTokenType);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LexErr {
    #[error("Unknown token '{0}' at {1}.")]
    UnknownToken(String, CharacterPosition),
}

/// Return if at end of reading character.
#[inline(always)]
pub fn is_end(c: char) -> bool {
    c == '\0'
}

#[inline]
pub fn is_break(c: char) -> bool {
    c == '\n' || c == '\r'
}

#[inline]
pub fn is_break_end(c: char) -> bool {
    is_break(c) || is_end(c)
}

#[inline]
pub fn is_blank(c: char) -> bool {
    c == ' ' || c == '\t'
}

#[inline]
pub fn is_blank_end(c: char) -> bool {
    is_blank(c) || is_break_end(c)
}

#[inline]
pub fn is_flow(c: char) -> bool {
    match c {
        ',' | '[' | ']' | '{' | '}' => true,
        _ => false,
    }
}
