////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Span tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Tokenize;
use crate::lexer::Lexer;
use crate::span::SpanPosition;
use crate::span::Page;


////////////////////////////////////////////////////////////////////////////////
// Token parser.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenError(&'static str);
impl std::error::Error for TokenError {}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum AtmaToken {
    CommandChunk,
    
    RawStringOpen,
    RawStringClose,
    RawStringText,
    
    StringOpenSingle,
    StringCloseSingle,
    StringOpenDouble,
    StringCloseDouble,
    StringText,

    LineCommentOpen,
    LineCommentText,
    
    Whitespace,
}

#[derive(Debug, Clone)]
struct AtmaScriptTokenizer {
    open: Option<AtmaToken>,
    raw_string_bracket_count: u8,
    newline_token: Box<str>,
}

impl AtmaScriptTokenizer {
    fn new() -> Self {
        AtmaScriptTokenizer {
            open: None,
            raw_string_bracket_count: 0,
            newline_token: "\n".into(),
        }
    }

    /// Parses a CommandChunk token.
    fn parse_command_chunk(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        let tok = text.split_whitespace().next().unwrap();
        if tok.len() > 0 {
            let span = SpanPosition::new_from_string(tok, &*self.newline_token);
            Some((AtmaToken::CommandChunk, span))
        } else {
            None
        }
    }

    /// Parses a RawStringOpen token.
    fn parse_raw_string_open(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        let mut pos = SpanPosition::ZERO;
        let mut chars = text.chars();
        match chars.next() {
            Some('r') => pos.step(1, 0, 1),
            _         => return None,
        }
        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    pos.step(1, 0, 1);
                    self.raw_string_bracket_count += 1;
                    continue;
                },
                '"' => {
                    pos.step(1, 0, 1);
                    return Some((AtmaToken::RawStringOpen, pos));
                },
                _ => return None,
            }
        }

        None
    }

    /// Parses a RawStringClose token.
    fn parse_raw_string_close(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        let mut raw_count = 0;
        let mut pos = SpanPosition::ZERO;
        let mut chars = text.chars();
        match chars.next() {
            Some('"') => pos.step(1, 0, 1),
            _         => return None,
        }
        while let Some(c) = chars.next() {
            match c {
                '#' => {
                    pos.step(1, 0, 1);
                    raw_count += 1;
                    continue;
                },
                _ => break,
            }
        }

        if self.raw_string_bracket_count == raw_count {
            self.raw_string_bracket_count = 0;
            Some((AtmaToken::RawStringClose, pos))
        } else {
            None
        }
    }


    /// Parses a RawStringText token.
    fn parse_raw_string_text(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        let mut pos = SpanPosition::ZERO;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '"' {
                return Some((AtmaToken::RawStringText, pos));
            }

            if c == '\n' {
                pos.step(1, 1, 0);
            } else {
                pos.step(c.len_utf8(), 0, 1);
            }
        }

        None
    }

    /// Parses a StringOpenSingle token.
    fn parse_string_open_single(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        if text.starts_with("'") {
            Some((AtmaToken::StringOpenSingle, SpanPosition::from(1)))
        } else {
            None
        }
    }
    
    /// Parses a StringCloseSingle token.
    fn parse_string_close_single(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        if text.starts_with("'") {
            Some((AtmaToken::StringCloseSingle, SpanPosition::from(1)))
        } else {
            None
        }
    }

    /// Parses a StringOpenDouble token.
    fn parse_string_open_double(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        if text.starts_with("\"") {
            Some((AtmaToken::StringOpenDouble, SpanPosition::from(1)))
        } else {
            None
        }
    }

    /// Parses a StringCloseDouble token.
    fn parse_string_close_double(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        if text.starts_with("\"") {
            Some((AtmaToken::StringCloseDouble, SpanPosition::from(1)))
        } else {
            None
        }
    }

    /// Parses a StringText token.
    fn parse_string_text(&mut self, text: &str, open: AtmaToken)
        -> Option<(AtmaToken, SpanPosition)>
    {
        let mut pos = SpanPosition::ZERO;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('\\') | 
                    Some('"')  | 
                    Some('\'') | 
                    Some('t')  | 
                    Some('r')  |
                    Some('n')  => pos.step(2, 0, 2),
                    Some('u')  => unimplemented!("unicode escapes unsupported"),
                    Some(_)    |
                    None       => return None,
                }
                continue;
            }

            match (c, open) {
                ('\'', AtmaToken::StringOpenSingle) |
                ('"', AtmaToken::StringOpenDouble)  => {
                    return Some((AtmaToken::StringText, pos));
                },
                ('\n', _) => pos.step(1, 1, 0),
                _         => pos.step(c.len_utf8(), 0, 1),
            }
        }

        None
    }

    /// Parses a LineCommentOpen token.
    fn parse_line_comment_open(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        if text.starts_with("#") {
            Some((AtmaToken::LineCommentOpen, SpanPosition::from(1)))
        } else {
            None
        }
    }

    /// Parses a LineCommentText token.
    fn parse_line_comment_text(&mut self, text: &str)
        -> (AtmaToken, SpanPosition)
    {
        if let Some(first) = text.split(&*self.newline_token).next() {
            let span = SpanPosition {
                byte: first.len(),
                page: Page { line: 0, column: first.len() },
            };
            (AtmaToken::LineCommentText, span)
        } else {
            unreachable!()
        }
    }

    /// Parses a Whitespace token.
    fn parse_whitespace(&mut self, text: &str)
        -> Option<(AtmaToken, SpanPosition)>
    {
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr_len = text.len() - rest.len();
            let span = SpanPosition::new_from_string(
                &text[0..substr_len],
                &*self.newline_token);
            Some((AtmaToken::Whitespace, span))
        } else {
            None
        }
    }
}

impl Tokenize for AtmaScriptTokenizer {
    type Token = AtmaToken;
    type Error = TokenError;

    fn parse_token<'text>(&mut self, text: &'text str)
        -> Result<(Self::Token, SpanPosition), (Self::Error, SpanPosition)>
    {
        use AtmaToken::*;
        match self.open.take() {
            Some(LineCommentOpen) => {
                Ok(self.parse_line_comment_text(text))
            },
            Some(RawStringText) => {
                if let Some(parse) = self.parse_raw_string_close(text) {
                    return Ok(parse);
                }
                Err((TokenError("Non-terminated raw string"), 0.into()))
            }
            Some(RawStringOpen) => {
                if let Some(parse) = self.parse_raw_string_close(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_raw_string_text(text)
                {
                    self.open = Some(RawStringText);
                    Ok(parse)
                } else {
                    Err((TokenError("Non-terminated raw string"), 0.into()))
                }
            },
            Some(StringOpenSingle) => {
                if let Some(parse) = self.parse_string_close_single(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self
                    .parse_string_text(text, StringOpenSingle)
                {
                    self.open = Some(StringOpenSingle);
                    Ok(parse)
                } else {
                    Err((TokenError("Unrecognized string escape"), 1.into()))
                }
            },
            Some(StringOpenDouble) => {
                if let Some(parse) = self.parse_string_close_double(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self
                    .parse_string_text(text, StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    Ok(parse)
                } else {
                    Err((TokenError("Unrecognized string escape"), 1.into()))
                }
            },

            None => {
                if let Some(parse) = self.parse_line_comment_open(text) {
                    self.open = Some(LineCommentOpen);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_raw_string_open(text) {
                    self.open = Some(RawStringOpen);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_string_open_single(text) {
                    self.open = Some(StringOpenSingle);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_string_open_double(text) {
                    self.open = Some(StringOpenDouble);
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_whitespace(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_command_chunk(text) {
                    return Ok(parse);
                }

                Err((TokenError("Unrecognized token"), 1.into()))
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////


/// Tests `Lexer::new` for the AtmaScriptTokenizer.
#[test]
fn as_lexer_empty() {
    let text = "";
    let as_tok = AtmaScriptTokenizer::new();
    let mut lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.next(),
        None);
}


/// Tests AtmaScriptTokenizer with a line comment.
#[test]
fn as_lexer_line_comment() {
    use AtmaToken::*;
    let text = "#abc\n";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (LineCommentOpen, "\"#\" (0:0-0:1, bytes 0-1)".to_owned()),
            (LineCommentText, "\"abc\" (0:1-0:4, bytes 1-4)".to_owned()),
            (Whitespace,      "\"\n\" (0:4-1:0, bytes 4-5)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with a line comment surrounded by whitespace.
#[test]
fn as_lexer_line_comment_circumfix_whitespace() {
    use AtmaToken::*;
    let text = "\n\t \n#abc\n \t\n ";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (Whitespace,      "\"\n\t \n\" (0:0-2:0, bytes 0-4)".to_owned()),
            (LineCommentOpen, "\"#\" (2:0-2:1, bytes 4-5)".to_owned()),
            (LineCommentText, "\"abc\" (2:1-2:4, bytes 5-8)".to_owned()),
            (Whitespace,      "\"\n \t\n \" (2:4-4:1, bytes 8-13)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with a line comment surrounded by whitespace.
#[test]
fn as_lexer_line_comments_remove_whitespace() {
    use AtmaToken::*;
    let text = "\n\t \n#abc\n \t#def\n ";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer
            .filter_map(|res| {
                let tok = res.unwrap();
                if tok.is_whitespace() { 
                    None
                } else {
                    Some((tok.value, format!("{}", tok.span)))
                }
            })
            .collect::<Vec<_>>(),
        vec![
            (LineCommentOpen, "\"#\" (2:0-2:1, bytes 4-5)".to_owned()),
            (LineCommentText, "\"abc\" (2:1-2:4, bytes 5-8)".to_owned()),
            (LineCommentOpen, "\"#\" (3:2-3:3, bytes 11-12)".to_owned()),
            (LineCommentText, "\"def\" (3:3-3:6, bytes 12-15)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with an empty single-quoted string.
#[test]
fn as_lexer_line_string_single_empty() {
    use AtmaToken::*;
    let text = "''";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
            (StringCloseSingle, "\"'\" (0:1-0:2, bytes 1-2)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with an unclosed single-quoted string.
#[test]
#[should_panic]
fn as_lexer_line_string_single_unclosed() {
    use AtmaToken::*;
    let text = "'abc";
    let as_tok = AtmaScriptTokenizer::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let tok = res.unwrap();
            (tok.value, format!("{}", tok.span))
        });

    assert_eq!(
        lexer.next(), 
        Some((StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((StringText,  "\"abc\" (0:1-0:4, bytes 1-4)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaScriptTokenizer with a non-empty single-quoted string.
#[test]
fn as_lexer_line_string_single_text() {
    use AtmaToken::*;
    let text = "'abc \n xyz'";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
            (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
            (StringCloseSingle, "\"'\" (1:4-1:5, bytes 10-11)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with a quote-containing single-quoted string.
#[test]
fn as_lexer_line_string_single_quotes() {
    use AtmaToken::*;
    let text = "'abc\"\n\\'xyz'";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
            (StringText,        "\"abc\"\n\\'xyz\" (0:1-1:5, bytes 1-11)".to_owned()),
            (StringCloseSingle, "\"'\" (1:5-1:6, bytes 11-12)".to_owned()),
        ]);
}


/// Tests AtmaScriptTokenizer with an empty double-quoted string.
#[test]
fn as_lexer_line_string_double_empty() {
    use AtmaToken::*;
    let text = "\"\"";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
            (StringCloseDouble, "\"\"\" (0:1-0:2, bytes 1-2)".to_owned()),
        ]);
}


/// Tests AtmaScriptTokenizer with an unclosed double-quoted string.
#[test]
#[should_panic]
fn as_lexer_line_string_double_unclosed() {
    use AtmaToken::*;
    let text = "\"abc";
    let as_tok = AtmaScriptTokenizer::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let tok = res.unwrap();
            (tok.value, format!("{}", tok.span))
        });

    assert_eq!(
        lexer.next(), 
        Some((StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((StringText,  "\"abc\" (0:1-0:4, bytes 1-4)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}


/// Tests AtmaScriptTokenizer with a non-empty double-quoted string.
#[test]
fn as_lexer_line_string_double_text() {
    use AtmaToken::*;
    let text = "\"abc \n xyz\"";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
            (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
            (StringCloseDouble, "\"\"\" (1:4-1:5, bytes 10-11)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with a quote-containing double-quoted string.
#[test]
fn as_lexer_line_string_double_quotes() {
    use AtmaToken::*;
    let text = "\"abc\\\"\n'xyz\"";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
            (StringText,        "\"abc\\\"\n'xyz\" (0:1-1:4, bytes 1-11)".to_owned()),
            (StringCloseDouble, "\"\"\" (1:4-1:5, bytes 11-12)".to_owned()),
        ]);
}








/// Tests AtmaScriptTokenizer with an empty raw-quoted string.
#[test]
fn as_lexer_line_string_raw_empty() {
    use AtmaToken::*;
    let text = "r\"\"";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (RawStringOpen,  "\"r\"\" (0:0-0:2, bytes 0-2)".to_owned()),
            (RawStringClose, "\"\"\" (0:2-0:3, bytes 2-3)".to_owned()),
        ]);
}


/// Tests AtmaScriptTokenizer with an empty raw-quoted string using hashes.
#[test]
fn as_lexer_line_string_raw_empty_hashed() {
    use AtmaToken::*;
    let text = "r##\"\"##";
    let as_tok = AtmaScriptTokenizer::new();
    let lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.map(|res| {
                let tok = res.unwrap();
                (tok.value, format!("{}", tok.span))
            })
            .collect::<Vec<_>>(),
        vec![
            (RawStringOpen,  "\"r##\"\" (0:0-0:4, bytes 0-4)".to_owned()),
            (RawStringClose, "\"\"##\" (0:4-0:7, bytes 4-7)".to_owned()),
        ]);
}

/// Tests AtmaScriptTokenizer with an unclosed raw-quoted string.
#[test]
#[should_panic]
fn as_lexer_line_string_raw_unclosed() {
    use AtmaToken::*;
    let text = "r###\"abc";
    let as_tok = AtmaScriptTokenizer::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let tok = res.unwrap();
            (tok.value, format!("{}", tok.span))
        });

    assert_eq!(
        lexer.next(), 
        Some((RawStringOpen,  "\"r###\"\" (0:0-0:5, bytes 0-5)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((RawStringText,  "\"abc\" (0:5-0:8, bytes 5-8)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaScriptTokenizer with an mismatched raw-quoted string.
#[test]
#[should_panic]
fn as_lexer_line_string_raw_mismatched() {
    use AtmaToken::*;
    let text = "r###\"abc\"#";
    let as_tok = AtmaScriptTokenizer::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let tok = res.unwrap();
            (tok.value, format!("{}", tok.span))
        });

    assert_eq!(
        lexer.next(), 
        Some((RawStringOpen,  "\"r###\"\" (0:0-0:5, bytes 0-5)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((RawStringText,  "\"abc\" (0:5-0:8, bytes 5-8)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

// /// Tests AtmaScriptTokenizer with a non-empty raw-quoted string.
// #[test]
// fn as_lexer_line_string_raw_text() {
//     use AtmaToken::*;
//     let text = "'abc \n xyz'";
//     let as_tok = AtmaScriptTokenizer::new();
//     let lexer = Lexer::new(as_tok, text);

//     assert_eq!(
//         lexer.map(|res| {
//                 let tok = res.unwrap();
//                 (tok.value, format!("{}", tok.span))
//             })
//             .collect::<Vec<_>>(),
//         vec![
//             (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
//             (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
//             (StringCloseSingle, "\"'\" (1:4-1:5, bytes 10-11)".to_owned()),
//         ]);
// }

// /// Tests AtmaScriptTokenizer with a quote-containing raw-quoted string.
// #[test]
// fn as_lexer_line_string_raw_quotes() {
//     use AtmaToken::*;
//     let text = "'abc\"\n\\'xyz'";
//     let as_tok = AtmaScriptTokenizer::new();
//     let lexer = Lexer::new(as_tok, text);

//     assert_eq!(
//         lexer.map(|res| {
//                 let tok = res.unwrap();
//                 (tok.value, format!("{}", tok.span))
//             })
//             .collect::<Vec<_>>(),
//         vec![
//             (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
//             (StringText,        "\"abc\"\n\\'xyz\" (0:1-1:5, bytes 1-11)".to_owned()),
//             (StringCloseSingle, "\"'\" (1:5-1:6, bytes 11-12)".to_owned()),
//         ]);
// }
