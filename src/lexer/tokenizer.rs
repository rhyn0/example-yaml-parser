use crate::{lexer::tokens, CharacterPosition, LexErr, YamlToken, YamlTokenType};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Lexer<T> {
    /// Source of characters
    reader: T,
    /// List of tokens that have been lexed
    tokens: VecDeque<tokens::YamlToken>,
    /// Reading buffer to perform operations on
    buffer: VecDeque<char>,
    /// current indent of last used block
    /// Can be negative at initialization as we won't know file's indentation level
    curr_indent: isize,
    /// YAML defines a flow syntax, which is a more explicit way of writing collections and scalars
    flow_level: u8,
    /// Store errors set caused by lexing.
    error: Option<LexErr>,
    stream_started: bool,
    stream_ended: bool,
    current_position: CharacterPosition,
    /// Whether we can return any tokens
    token_available: bool,
}

pub type LexResult = Result<(), tokens::LexErr>;

impl<T: Iterator<Item = char>> Lexer<T> {
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            buffer: VecDeque::new(),
            tokens: VecDeque::new(),
            error: None,
            stream_ended: false,
            stream_started: false,
            current_position: CharacterPosition::default(),
            curr_indent: -1,
            flow_level: 0,
            token_available: false,
        }
    }
    // ## methods to move past characters ##
    fn first_char(&self) -> char {
        self.buffer[0]
    }
    fn skip(&mut self) {
        let c = self.buffer.pop_front().unwrap();
        self.current_position.next_index();
        if c == '\n' {
            self.current_position.column = 0;
            self.current_position.line += 1;
        } else {
            self.current_position.column += 1;
        }
    }
    /// Skip a CRLF or LF
    fn skip_break(&mut self) {
        let c = self.buffer[0];
        let nc = self.buffer[1];
        if c == '\r' && nc == '\n' {
            self.skip();
            self.skip();
        } else if tokens::is_break(c) {
            self.skip();
        }
    }
    fn ensure_chars(&mut self, num_chars: usize) {
        if self.buffer.len() >= num_chars {
            return;
        }
        for _ in 0..num_chars - self.buffer.len() {
            self.buffer.push_back(self.reader.next().unwrap_or('\0'));
        }
    }
    /// certain control characters are of length 4 e.g. '...' <- newline
    fn ensure_control_chars_len(&mut self) {
        self.ensure_chars(4);
    }
    fn skip_to_token(&mut self) {
        loop {
            // make sure we can peek next character at least
            self.ensure_chars(1);
            match self.first_char() {
                ' ' | '\t' => self.skip(),
                '\n' | '\r' => {
                    self.ensure_chars(2);
                    self.skip_break();
                }
                '#' => {
                    // read through comment until back to normal
                    // comment takes rest of line, so either it ends or we newline
                    while !tokens::is_break_end(self.first_char()) {
                        self.skip();
                        self.ensure_chars(1);
                    }
                }
                _ => break, // real character
            }
        }
    }
    #[inline]
    fn read_break(&mut self, s: &mut String) {
        if self.buffer[0] == '\r' && self.buffer[1] == '\n' {
            s.push('\n');
            self.skip();
            self.skip();
        } else if self.buffer[0] == '\r' || self.buffer[0] == '\n' {
            s.push('\n');
            self.skip();
        } else {
            unreachable!();
        }
    }
    // ## Iteration and lazy evaluation functions
    fn next_token(&mut self) -> Result<Option<YamlToken>, LexErr> {
        if self.stream_ended {
            return Ok(None);
        }

        if !self.token_available {
            self.fetch_token()?;
        }

        let tok = self.tokens.pop_front().unwrap();
        if self.tokens.len() == 0 {
            self.token_available = false;
        }
        if let YamlTokenType::End = tok.1 {
            self.stream_ended = true;
        }
        Ok(Some(tok))
    }
    /// Insert a token at a specific spot in the array.
    /// Args:
    ///     position (usize): steps from end of array to insert it.
    fn insert_token(&mut self, position: usize, token: YamlToken) {
        let insert_idx = self.tokens.len().saturating_sub(position + 1);
        self.tokens.insert(insert_idx, token);
    }
    // All Fetch functions must make the self determination of whether a token is ready now or not
    fn fetch_token(&mut self) -> LexResult {
        let mut continue_loop;
        loop {
            continue_loop = false;
            if !self.token_available {
                continue_loop = true;
            }
            // there is this weird line because I imagine there will be some extra logic here
            if !continue_loop {
                break;
            }
            self.fetch_next_tok()?;
        }
        Ok(())
    }
    fn start_stream(&mut self) -> LexResult {
        let pos = self.current_position;
        self.stream_started = true;
        self.curr_indent = -1;
        self.token_available = true;
        self.tokens.push_back(YamlToken(pos, YamlTokenType::Start));
        Ok(())
    }
    fn end_stream(&mut self) -> LexResult {
        if self.current_position.column != 0 {
            self.current_position.column = 0;
            self.current_position.line += 1;
        }
        self.token_available = true;
        self.tokens
            .push_back(YamlToken(self.current_position, YamlTokenType::End));
        Ok(())
    }
    fn fetch_next_tok(&mut self) -> LexResult {
        self.ensure_chars(1);

        if !self.stream_started {
            return self.start_stream();
        }

        // unknown amount of whitespace between now and real characters
        self.skip_to_token();

        // remember where our position is
        let pos = self.current_position;

        self.ensure_control_chars_len();

        // hit end somehow
        if tokens::is_end(self.first_char()) {
            return self.end_stream();
        }

        let c = self.buffer[0];
        let nc = self.buffer[1];
        match c {
            ':' if tokens::is_blank_end(nc) || (self.flow_level > 0 && tokens::is_flow(nc)) => {
                self.fetch_value()
            }
            _ => self.fetch_scalar(),
        }
    }
    // ## Specfic Token Fetchers ##
    /// Read a simple scalar value - leave as string representation.
    /// A scalar value is not a token until we parse some more and learn what it is
    fn fetch_scalar(&mut self) -> LexResult {
        let tok = self.read_scalar()?;
        self.tokens.push_back(tok);
        Ok(())
    }
    fn fetch_value(&mut self) -> LexResult {
        // handle key and value based on observation of ':' from prior.
        let prev_scalar = self.tokens.get(self.tokens.len() - 1).unwrap();
        // scalar key should have been prior
        self.insert_token(1, YamlToken(prev_scalar.0, YamlTokenType::Key));

        // skip the ': '
        self.skip();
        self.skip();

        self.tokens
            .push_back(YamlToken(self.current_position, YamlTokenType::Value));
        self.token_available = true;
        self.fetch_scalar()
    }
    // ## Specific Token Readers. Deal with buffer ##
    fn read_scalar(&mut self) -> Result<YamlToken, LexErr> {
        let indent = self.curr_indent + 1;
        let pos = self.current_position;

        let mut trailing_breaks = String::new();
        let mut scalar = String::new();
        loop {
            // various non scalar checks to force us to exit
            self.ensure_control_chars_len();
            // is docstring
            if self.first_char() == '#' {
                break;
            }

            // build string version of value
            while !tokens::is_blank_end(self.first_char()) {
                // use up to first two characters to test if we ended our value
                match self.buffer[0] {
                    ':' if tokens::is_blank_end(self.buffer[1]) => break,
                    _ => {}
                }
                scalar.push(self.first_char());
                self.skip();
                self.ensure_chars(2);
            }
            // did we hit end of stream
            // while only breaks above if we encounter blank, break or end
            if tokens::is_end(self.first_char()) {
                break;
            }
            self.ensure_chars(1);
            // if blank or break, we need to add on to the string
            while tokens::is_blank(self.first_char()) || tokens::is_break(self.first_char()) {
                if tokens::is_blank(self.first_char()) {
                    self.skip();
                } else {
                    self.read_break(&mut trailing_breaks);
                }
                self.ensure_chars(1);
            }
            if self.flow_level == 0 || (pos.column as isize) < indent {
                break;
            }
        }
        Ok(YamlToken(pos, YamlTokenType::Scalar(scalar)))
    }
}

impl<T: Iterator<Item = char>> Iterator for Lexer<T> {
    type Item = YamlToken;
    fn next(&mut self) -> Option<YamlToken> {
        if self.error.is_some() {
            return None;
        }
        eprintln!("no error, next_token");
        match self.next_token() {
            Ok(tok) => tok,
            Err(e) => {
                self.error = Some(e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Borrowed from YAML Rust
    // https://github.com/chyh1990/yaml-rust/blob/da52a68615f2ecdd6b7e4567019f280c433c1521/src/scanner.rs#L1746
    macro_rules! next {
        ($p:ident, $tk:pat) => {{
            let thing = $p.next();
            let tok = thing.unwrap();
            match tok.1 {
                $tk => {
                    println!("matched token: {:?}", tok)
                }
                _ => panic!("unexpected token: {:?}", tok),
            }
        }};
    }

    #[test]
    fn test_simple_kv() {
        let s = "
        key1: value1
        key2: value2
        ";
        // result should have 2 Keys and 2 Values
        let mut lexer = Lexer::new(s.chars());
        next!(lexer, YamlTokenType::Start);
        next!(lexer, YamlTokenType::Key);
        next!(lexer, YamlTokenType::Scalar(_));
        next!(lexer, YamlTokenType::Value);
        next!(lexer, YamlTokenType::Scalar(_));
        next!(lexer, YamlTokenType::Key);
        next!(lexer, YamlTokenType::Scalar(_));
        next!(lexer, YamlTokenType::Value);
        next!(lexer, YamlTokenType::Scalar(_));
        next!(lexer, YamlTokenType::End);
        assert_eq!(lexer.tokens.len(), 0);
    }
    #[test]
    #[should_panic]
    fn test_bad_yaml() {
        let s = "
        key1 - value1
        ";
        let mut lexer = Lexer::new(s.chars());
        next!(lexer, YamlTokenType::Start);
        next!(lexer, YamlTokenType::Key);
    }
}
