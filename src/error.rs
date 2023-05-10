use super::Stream;
use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Range, RangeInclusive},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorMessage {
    Text(String),
    UnexpectedEOF,
    Expected(Expected),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expected {
    Char(char),
    Str(&'static str),
    OneOf(&'static str),
    Range(Range<char>),
    RangeInclusive(RangeInclusive<char>),
    Rule(&'static str),
}

impl From<String> for ErrorMessage {
    fn from(value: String) -> Self {
        ErrorMessage::Text(value)
    }
}

impl From<Expected> for ErrorMessage {
    fn from(value: Expected) -> Self {
        ErrorMessage::Expected(value)
    }
}

impl Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expected::Char(c) => {
                if *c == '\0' {
                    write!(f, "EOF")
                } else {
                    write!(f, "{c:?}")
                }
            }
            Expected::Str(s) => write!(f, "{s:?}"),
            Expected::OneOf(v) => {
                write!(f, "one of {:?}", v.chars().collect::<Vec<_>>())
            }
            Expected::Range(r) => write!(f, "{r:?}"),
            Expected::RangeInclusive(r) => write!(f, "{r:?}"),
            Expected::Rule(r) => write!(f, "<{r}>"),
        }
    }
}
impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorMessage::Text(s) => write!(f, "{}", s),
            ErrorMessage::UnexpectedEOF => write!(f, "unexpected EOF"),
            ErrorMessage::Expected(e) => write!(f, "expected {e}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error<'i> {
    pub stream: Stream<'i>,
    pub messages: HashSet<ErrorMessage>,
}

impl<'i> Error<'i> {
    #[inline(always)]
    pub fn new(stream: Stream<'i>, message: ErrorMessage) -> Self {
        let mut set = HashSet::new();
        set.insert(message);
        Error {
            stream,
            messages: set,
        }
    }

    pub fn message(&self) -> String {
        let mut expected = Vec::with_capacity(self.messages.len());
        let mut other = Vec::with_capacity(self.messages.len());
        for message in self.messages.iter() {
            if let ErrorMessage::Expected(e) = message {
                expected.push(e.to_string());
            } else {
                other.push(message.to_string());
            }
        }
        if !expected.is_empty() {
            let message = if expected.len() == 1 {
                format!("expected {}", expected[0])
            } else {
                format!("expected ({})", expected.join(" | "))
            };
            other.push(message)
        }
        other.join(" | ")
    }

    pub fn or(mut self, error: Error<'i>) -> Error<'i> {
        if self.stream.rest_len() == error.stream.rest_len() {
            self.messages.extend(error.messages);
            self
        } else if self.stream.rest_len() < error.stream.rest_len() {
            self
        } else {
            error
        }
    }
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for Error<'_> {}

pub type PResult<'i, R> = Result<(Stream<'i>, R), Error<'i>>;
