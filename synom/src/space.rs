use {IResult, ParseState};
use unicode_xid::UnicodeXID;

pub fn whitespace(input: ParseState) -> IResult<ParseState, ()> {
    if input.is_empty() {
        return IResult::Error;
    }

    let bytes = input.rest().as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let s = input.advance(i);
        if bytes[i] == b'/' {
            if s.starts_with("//") && (!s.starts_with("///") || s.starts_with("////")) &&
               !s.starts_with("//!") {
                if let Some(len) = s.rest().find('\n') {
                    i += len + 1;
                    continue;
                }
                break;
            } else if s.starts_with("/*") && (!s.starts_with("/**") || s.starts_with("/***")) &&
                      !s.starts_with("/*!") {
                match block_comment(s) {
                    IResult::Done(_, com) => {
                        i += com.len();
                        continue;
                    }
                    IResult::Error => {
                        return IResult::Error;
                    }
                }
            }
        }
        match bytes[i] {
            b' ' | 0x09...0x0d => {
                i += 1;
                continue;
            }
            b if b <= 0x7f => {}
            _ => {
                let ch = s.chars().next().unwrap();
                if is_whitespace(ch) {
                    i += ch.len_utf8();
                    continue;
                }
            }
        }
        return if i > 0 {
            IResult::Done(s, ())
        } else {
            IResult::Error
        };
    }
    IResult::Done(input.finish(), ())
}

pub fn block_comment(input: ParseState) -> IResult<ParseState, &str> {
    if !input.starts_with("/*") {
        return IResult::Error;
    }

    let mut depth = 0;
    let bytes = input.rest().as_bytes();
    let mut i = 0;
    let upper = bytes.len() - 1;
    while i < upper {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 1; // eat '*'
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            if depth == 0 {
                return IResult::Done(input.advance(i + 2), input.until(i + 2));
            }
            i += 1; // eat '/'
        }
        i += 1;
    }
    IResult::Error
}

pub fn word_break(input: ParseState) -> IResult<ParseState, ()> {
    match input.chars().next() {
        Some(ch) if UnicodeXID::is_xid_continue(ch) => IResult::Error,
        Some(_) | None => IResult::Done(input, ()),
    }
}

pub fn skip_whitespace(input: ParseState) -> ParseState {
    match whitespace(input) {
        IResult::Done(rest, _) => rest,
        IResult::Error => input,
    }
}

fn is_whitespace(ch: char) -> bool {
    // Rust treats left-to-right mark and right-to-left mark as whitespace
    ch.is_whitespace() || ch == '\u{200e}' || ch == '\u{200f}'
}
