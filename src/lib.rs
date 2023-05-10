pub mod common;
mod context;
mod error;
mod parser;
mod parsers;
mod stream;

pub use error::{Error, ErrorMessage, Expected, PResult};
pub use parser::Parser;
pub use parsers::{one_of, Any, EOF};
pub use stream::Stream;

#[macro_export]
macro_rules! parser {
    ($name: ident -> $result: ty = $body: expr ) => {
        fn $name<'i>(stream: Stream<'i>) -> PResult<'i, $result> {
            $body.parse(stream)
        }
    };
}

#[macro_export]
macro_rules! bin_op {
    ($cons: tt, $result: ty, $primary: expr) => {
        $primary
    };
    ($cons: tt, $result: ty, $op: expr, $next: expr) => {{
        fn bin_op_parser<'i>(stream: Stream<'i>) -> PResult<'i, $result> {
            $next
                .seq($op.seq($next).many())
                .map(|(mut left, rest)| {
                    for (op, right) in rest {
                        left = $cons!(left, op, right);
                    }
                    left
                })
                .parse(stream)
        }
        bin_op_parser
    }};
    ($cons: tt, $result: ty, $first: expr, $($rest: expr),*) => {
        bin_op!(
            $cons,
            $result,
            $first,
            bin_op!($cons, $result, $($rest),*)
        )
    };
}
