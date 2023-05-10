use super::{parser::Parser, ErrorMessage, Expected, PResult, Stream};
use std::ops::{Range, RangeInclusive};

impl<'i, R, F: Clone + Fn(Stream<'i>) -> PResult<'i, R>> Parser<'i> for F {
    type Result = R;

    #[inline(always)]
    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        self(stream)
    }
}

impl<'i, R, C: Clone, F: Clone + Fn(C, Stream<'i>) -> PResult<'i, R>> Parser<'i> for (C, F) {
    type Result = R;

    #[inline(always)]
    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let (ctx, func) = self;
        func(ctx.clone(), stream)
    }
}

impl<'i> Parser<'i> for char {
    type Result = char;

    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let (end, ch) = stream.next();
        if ch == *self {
            end.ok(ch)
        } else {
            stream.err(Expected::Char(*self).into())
        }
    }
}

impl<'i> Parser<'i> for &'static str {
    type Result = &'static str;

    fn parse(&self, mut stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let start = stream.clone();
        for ch in self.chars() {
            let (s, c) = stream.next();
            stream = s;
            if ch != c {
                return start.err(Expected::Str(*self).into());
            }
        }
        stream.ok(self)
    }
}

impl<'i> Parser<'i> for Range<char> {
    type Result = char;

    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let (end, ch) = stream.next();
        if self.start <= ch && ch < self.end {
            end.ok(ch)
        } else {
            stream.err(Expected::Range(self.clone()).into())
        }
    }
}

impl<'i> Parser<'i> for RangeInclusive<char> {
    type Result = char;

    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let (end, ch) = stream.next();
        if *self.start() <= ch && ch <= *self.end() {
            end.ok(ch)
        } else {
            stream.err(Expected::RangeInclusive(self.clone()).into())
        }
    }
}

#[derive(Clone)]
pub struct OneOf(Vec<char>, &'static str);

impl<'i> Parser<'i> for OneOf {
    type Result = char;

    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let (end, ch) = stream.next();
        if self.0.contains(&ch) {
            end.ok(ch)
        } else {
            stream.err(Expected::OneOf(self.1).into())
        }
    }
}

pub fn one_of(chars: &'static str) -> OneOf {
    OneOf(chars.chars().collect(), chars)
}

pub const EOF: char = '\0';

#[derive(Clone, Copy)]
pub struct Any;

impl<'i> Parser<'i> for Any {
    type Result = char;

    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result> {
        let (end, ch) = stream.next();
        if ch != EOF {
            end.ok(ch)
        } else {
            stream.err(ErrorMessage::UnexpectedEOF)
        }
    }
}
