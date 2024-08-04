use crate::{lexer::tokens::CharacterPosition, Lexer, YamlToken, YamlTokenType};

/// state machine moves from state to state and
/// expects certain token types at each state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    StreamStart,
}

/// `Event` is used with the low-level event base parsing API,
/// see `EventReceiver` trait.
#[derive(Clone, PartialEq, Debug, Eq)]
pub enum Event {
    /// Reserved for internal use
    Nothing,
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    /// Refer to an anchor ID
    Alias(usize),
    /// Value, style, anchor_id, tag
    Scalar(String, usize, Option<YamlTokenType>),
    /// Anchor ID
    SequenceStart(usize),
    SequenceEnd,
    /// Anchor ID
    MappingStart(usize),
    MappingEnd,
}

#[derive(Debug)]
pub struct Parser<T> {
    scanner: Lexer<T>,
    states: Vec<State>,
    state: State,
    token: Option<YamlToken>,
    current: Option<(Event, CharacterPosition)>,
}
