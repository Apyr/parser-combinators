use super::{Error, ErrorMessage};
use std::{
    cell::RefCell,
    collections::HashSet,
    mem::{swap, take},
    str::Chars,
};

#[derive(Debug, Clone)]
pub(super) struct Context<'i> {
    pub text: &'i str,
    pub catcher: RefCell<Catcher<'i>>,
}

impl<'i> Context<'i> {
    pub fn new(text: &'i str) -> Context<'i> {
        Context {
            text,
            catcher: Catcher::new(text.chars()).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Catcher<'i> {
    chars: Chars<'i>,
    messages: HashSet<ErrorMessage>,
    pub is_started: bool,
}

impl<'i> Catcher<'i> {
    fn new(chars: Chars<'i>) -> Catcher<'i> {
        Catcher {
            chars,
            messages: HashSet::new(),
            is_started: true,
        }
    }

    #[inline(always)]
    pub fn set_started(&mut self, is_started: bool) -> bool {
        let old = self.is_started;
        self.is_started = is_started;
        old
    }

    pub fn pop_error(&mut self, mut start_chars: Chars<'i>) -> (Chars<'i>, HashSet<ErrorMessage>) {
        let messages = take(&mut self.messages);
        swap(&mut self.chars, &mut start_chars);
        (start_chars, messages)
    }

    pub fn set_error(&mut self, err: Error<'i>) {
        self.messages = err.messages;
        self.chars = err.stream.chars;
    }
}
