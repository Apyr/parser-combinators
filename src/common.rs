use super::{one_of, Any, PResult, Parser, Stream, EOF};

pub fn ws<'i>(stream: Stream<'i>) -> PResult<'i, ()> {
    let spaces = one_of(" \n\r\t").many().rule("ws");
    spaces.map(|_| ()).parse(stream)
}

pub fn ident<'i>(stream: Stream<'i>) -> PResult<'i, String> {
    let letter = ('a'..='z').or('A'..='Z').or('_');
    let letter_or_digit = letter.or('0'..='9');
    let ident = letter
        .prepend(letter_or_digit.many())
        .as_string()
        .rule("ident");
    ident.parse(stream)
}

fn escape<'i>(stream: Stream<'i>) -> PResult<'i, char> {
    fn escape_code<'i>(stream: Stream<'i>) -> PResult<'i, char> {
        let digit = ('0'..='9').or('a'..='f').or('A'..='F');
        let p = "\\x"
            .ignore_prev(digit.in_range(2..=2))
            .or("\\u{".ignore_prev(digit.in_range(1..=6)).ignore_this('}'));
        p.parse(stream.clone()).and_then(|(s, digits)| {
            let code = u32::from_str_radix(&String::from_iter(digits), 16).unwrap();
            if let Some(c) = char::from_u32(code) {
                s.ok(c)
            } else {
                stream.err(format!("invalid unicode code {:x}", code).into())
            }
        })
    }

    let p = "\\n"
        .map(|_| '\n')
        .or("\\0".map(|_| '\0'))
        .or("\\r".map(|_| '\r'))
        .or("\\t".map(|_| '\t'))
        .or("\\\\".map(|_| '\\'))
        .or(escape_code);
    p.rule("escape").parse(stream)
}

pub fn string<'i>(stream: Stream<'i>) -> PResult<'i, String> {
    let ch = escape
        .or("\\\"".map(|_| '"'))
        .or(Any.and_not(EOF).and_not('"'));
    let str = '"'.ignore_prev(ch.many().ignore_this('"'));
    let str = str.as_string().rule("string");
    str.parse(stream)
}

pub fn character<'i>(stream: Stream<'i>) -> PResult<'i, char> {
    let ch = escape
        .or("\\'".map(|_| '\''))
        .or(Any.and_not(EOF).and_not('\''));
    let char = '\''.ignore_prev(ch.ignore_this('\'')).rule("character");
    char.parse(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_idents() {
        let text = "abc d12 _e";
        fn idents<'i>(stream: Stream<'i>) -> PResult<'i, Vec<String>> {
            let r = ws.ignore_prev(ident).many().ignore_this(EOF);
            r.parse(stream)
        }
        let result = idents(Stream::new(text));
        assert_eq!(
            result.map(|(_, r)| r),
            Ok(vec!["abc".to_string(), "d12".to_string(), "_e".to_string(),])
        );
    }

    #[test]
    fn parse_string() {
        let text = r#"
            "Hello, world\n"
        "#;
        let result = ws
            .ignore_prev(string)
            .parse(Stream::new(text))
            .map(|(_, r)| r);
        assert_eq!(result, Ok("Hello, world\n".to_string()));
    }
}
