use super::{Expected, PResult, Stream};
use std::{fmt::Debug, ops::RangeBounds};

type CtxFn<'i, C, R> = (C, fn(C, Stream<'i>) -> PResult<'i, R>);

pub trait Parser<'i>: Clone {
    type Result;

    fn parse(&self, stream: Stream<'i>) -> PResult<'i, Self::Result>;

    #[inline(always)]
    fn or<P: Parser<'i, Result = Self::Result>>(
        &self,
        other: P,
    ) -> CtxFn<'i, (Self, P), Self::Result> {
        let ctx = (self.clone(), other);
        (ctx, |(p1, p2), stream| match p1.parse(stream.clone()) {
            r @ Ok(_) => r,
            Err(err1) => match p2.parse(stream) {
                r @ Ok(_) => r,
                Err(err2) => {
                    let stream = err1.stream.clone();
                    let err = stream.catch(err1.or(err2));
                    Err(err)
                }
            },
        })
    }

    #[inline(always)]
    fn seq<P: Parser<'i>>(&self, other: P) -> CtxFn<'i, (Self, P), (Self::Result, P::Result)> {
        let ctx = (self.clone(), other);
        (ctx, |(p1, p2), stream| {
            let (stream, r1) = p1.parse(stream)?;
            let (stream, r2) = p2.parse(stream)?;
            stream.ok((r1, r2))
        })
    }

    #[inline(always)]
    fn and_not<P: Debug + Parser<'i>>(&self, other: P) -> CtxFn<'i, (Self, P), Self::Result> {
        let ctx = (self.clone(), other);
        (ctx, |(p1, p2), stream| {
            let (s, r) = p1.parse(stream.clone())?;
            if p2.parse(stream.clone()).is_ok() {
                stream.err(format!("unexpected {:?}", p2).into())
            } else {
                s.ok(r)
            }
        })
    }

    #[inline(always)]
    fn prepend<P: Parser<'i, Result = Vec<Self::Result>>>(
        &self,
        other: P,
    ) -> CtxFn<'i, (Self, P), Vec<Self::Result>> {
        let ctx = (self.clone(), other);
        (ctx, |(p1, p2), stream| {
            let (stream, r1) = p1.parse(stream)?;
            let (stream, mut r2) = p2.parse(stream)?;
            r2.insert(0, r1);
            stream.ok(r2)
        })
    }

    #[inline(always)]
    fn opt(&self) -> CtxFn<'i, Self, Option<Self::Result>> {
        (self.clone(), |p, stream| match p.parse(stream.clone()) {
            Ok((s, r)) => s.ok(Some(r)),
            Err(err) => {
                stream.catch(err);
                stream.ok(None)
            }
        })
    }

    #[inline(always)]
    fn many(&self) -> CtxFn<'i, Self, Vec<Self::Result>> {
        (self.clone(), |p, mut stream| {
            let mut result = vec![];
            loop {
                match p.parse(stream.clone()) {
                    Ok((s, r)) => {
                        stream = s;
                        result.push(r);
                    }
                    Err(err) => {
                        stream.catch(err);
                        break;
                    }
                }
            }
            stream.ok(result)
        })
    }

    #[inline(always)]
    fn in_range<R: Debug + Clone + RangeBounds<usize>>(
        &self,
        range: R,
    ) -> CtxFn<'i, (Self, R), Vec<Self::Result>> {
        let ctx = (self.clone(), range);
        (ctx, |(p, range), stream| {
            let (s, r) = p.many().parse(stream.clone())?;
            if range.contains(&r.len()) {
                s.ok(r)
            } else {
                stream.err(format!("not in range {:?}", range).into())
            }
        })
    }

    #[inline(always)]
    fn map<R, F: Clone + Fn(Self::Result) -> R>(&self, func: F) -> CtxFn<'i, (Self, F), R> {
        let ctx = (self.clone(), func);
        (ctx, |(p, func), stream| {
            p.parse(stream).map(|(s, r)| (s, func(r)))
        })
    }

    #[inline(always)]
    fn as_string(&self) -> CtxFn<'i, Self, String>
    where
        Self::Result: IntoIterator<Item = char>,
    {
        (self.clone(), |p, stream| {
            p.parse(stream).map(|(s, r)| (s, String::from_iter(r)))
        })
    }

    #[inline(always)]
    fn some(&self) -> CtxFn<'i, Self, Vec<Self::Result>> {
        (self.clone(), |p, stream| p.prepend(p.many()).parse(stream))
    }

    #[inline(always)]
    fn ignore_prev<P: Parser<'i>>(&self, other: P) -> CtxFn<'i, (Self, P), P::Result> {
        let ctx = (self.clone(), other);
        (ctx, |(p1, p2), stream| {
            let (stream, _) = p1.parse(stream)?;
            let (stream, r2) = p2.parse(stream)?;
            stream.ok(r2)
        })
    }

    #[inline(always)]
    fn ignore_this<P: Parser<'i>>(&self, other: P) -> CtxFn<'i, (Self, P), Self::Result> {
        let ctx = (self.clone(), other);
        (ctx, |(p1, p2), stream| {
            let (stream, r1) = p1.parse(stream)?;
            let (stream, _) = p2.parse(stream)?;
            stream.ok(r1)
        })
    }

    #[inline(always)]
    fn opt_default(&self) -> CtxFn<'i, Self, Self::Result>
    where
        Self::Result: Default,
    {
        (self.clone(), |p, stream| {
            let (stream, r) = p.opt().parse(stream)?;
            stream.ok(r.unwrap_or_default())
        })
    }

    #[inline(always)]
    fn list<S: Parser<'i>>(&self, sep: S) -> CtxFn<'i, (Self, S), Vec<Self::Result>> {
        let ctx = (self.clone(), sep);
        (ctx, |(p, sep), stream| {
            let list_parser = p.prepend(sep.ignore_prev(p.clone()).many());
            list_parser.parse(stream)
        })
    }

    #[inline(always)]
    fn list_trailing<S: Parser<'i>>(&self, sep: S) -> CtxFn<'i, (Self, S), Vec<Self::Result>> {
        let ctx = (self.clone(), sep);
        (ctx, |(p, sep), stream| {
            let list_parser = p
                .prepend(sep.ignore_prev(p.clone()).many())
                .ignore_this(sep.opt());
            list_parser.parse(stream)
        })
    }

    #[inline(always)]
    fn rule(&self, rule: &'static str) -> CtxFn<'i, (Self, &'static str), Self::Result> {
        let ctx = (self.clone(), rule);
        (ctx, |(p, rule), stream| {
            let ctx = stream.ctx.clone();
            let is_started = ctx.catcher.borrow_mut().set_started(false);
            let result = p.parse(stream).map_err(|mut err| {
                err.messages.clear();
                err.messages.insert(Expected::Rule(rule).into());
                err
            });
            ctx.catcher.borrow_mut().set_started(is_started);
            result
        })
    }
}
