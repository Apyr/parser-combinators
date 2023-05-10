use super::{context::Context, Error, ErrorMessage, PResult};
use std::{fmt::Debug, rc::Rc, str::Chars};

#[derive(Clone)]
pub struct Stream<'i> {
    pub(super) chars: Chars<'i>,
    pub(super) ctx: Rc<Context<'i>>,
}

impl Debug for Stream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Stream").field(&self.chars.as_str()).finish()
    }
}

impl PartialEq for Stream<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.chars.as_str() == other.chars.as_str()
    }
}

impl Eq for Stream<'_> {}

impl<'i> Stream<'i> {
    pub fn new(text: &'i str) -> Stream<'i> {
        let ctx = Context::new(text);
        Stream {
            chars: text.chars(),
            ctx: Rc::new(ctx),
        }
    }

    #[inline(always)]
    pub fn rest_len(&self) -> usize {
        self.chars.as_str().len()
    }

    pub fn next(&self) -> (Stream<'i>, char) {
        let mut chars = self.chars.clone();
        let ch = chars.next().unwrap_or('\0');
        (
            Stream {
                chars,
                ctx: self.ctx.clone(),
            },
            ch,
        )
    }

    #[inline(always)]
    pub fn ok<R>(&self, result: R) -> PResult<'i, R> {
        Ok((self.clone(), result))
    }

    #[inline(always)]
    pub fn err<R>(&self, message: ErrorMessage) -> PResult<'i, R> {
        let err = Error::new(self.clone(), message);
        let err = self.catch(err);
        Err(err)
    }

    pub fn catch(&self, error: Error<'i>) -> Error<'i> {
        let mut catcher = self.ctx.catcher.borrow_mut();
        if !catcher.is_started {
            return error;
        }
        let (chars, messages) = catcher.pop_error(self.ctx.text.chars());
        let mut stream = self.clone();
        stream.chars = chars;
        let err = error.or(Error { stream, messages });
        catcher.set_error(err.clone());
        err
    }

    /*pub fn clear_error(&self) {
        let mut catcher = self.ctx.catcher.borrow_mut();
        if catcher.rest_len() > self.rest_len() {
            let _ = catcher.pop_error(self.ctx.text.chars());
        }
    }*/
}
