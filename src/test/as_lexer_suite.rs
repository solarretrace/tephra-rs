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
use crate::lexer::Scanner;
use crate::lexer::Lexer;
use crate::span::Pos;
use crate::span::Page;

// Standard library imports.
use std::convert::TryInto as _;

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
    CommandTerminator,
    
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
struct AtmaScriptScannerr {
    open: Option<AtmaToken>,
    newline_token: Box<str>,
    raw_string_bracket_count: u64,
}

impl AtmaScriptScannerr {
    fn new() -> Self {
        AtmaScriptScannerr {
            open: None,
            raw_string_bracket_count: 0,
            newline_token: "\n".into(),
        }
    }

    /// Parses a CommandChunk token.
    fn parse_command_chunk(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let tok = text
            .split(|c: char| c.is_whitespace() || c == ';')
            .next().unwrap();
        if tok.len() > 0 {
            let pos = Pos::new_from_string(tok, &*self.newline_token);
            Some((AtmaToken::CommandChunk, pos))
        } else {
            None
        }
    }

    /// Parses a CommandTerminator token.
    fn parse_command_terminator(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with(";") {
            Some((AtmaToken::CommandTerminator, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a RawStringOpen token.
    fn parse_raw_string_open(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let mut pos = Pos::ZERO;
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
        -> Option<(AtmaToken, Pos)>
    {
        let mut raw_count = 0;
        let mut pos = Pos::ZERO;
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
                    if raw_count >= self.raw_string_bracket_count {
                        break;
                    }
                },
                _ => break,
            }
        }

        if self.raw_string_bracket_count == raw_count {
            Some((AtmaToken::RawStringClose, pos))
        } else {
            None
        }
    }

    /// Parses a RawStringText token.
    fn parse_raw_string_text(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '"' {
                if self.parse_raw_string_close(&text[pos.byte..]).is_some() {
                    self.open = Some(AtmaToken::RawStringText);
                    return Some((AtmaToken::RawStringText, pos));
                }
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
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("'") {
            Some((AtmaToken::StringOpenSingle, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }
    
    /// Parses a StringCloseSingle token.
    fn parse_string_close_single(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("'") {
            Some((AtmaToken::StringCloseSingle, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a StringOpenDouble token.
    fn parse_string_open_double(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("\"") {
            Some((AtmaToken::StringOpenDouble, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a StringCloseDouble token.
    fn parse_string_close_double(&mut self, text: &str)
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("\"") {
            Some((AtmaToken::StringCloseDouble, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a StringText token.
    fn parse_string_text(&mut self, text: &str, open: AtmaToken)
        -> Option<(AtmaToken, Pos)>
    {
        let mut pos = Pos::ZERO;
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    // NOTE: These should all step by column, because they're
                    // escaped text.
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
        -> Option<(AtmaToken, Pos)>
    {
        if text.starts_with("#") {
            Some((AtmaToken::LineCommentOpen, Pos::new(1, 0, 1)))
        } else {
            None
        }
    }

    /// Parses a LineCommentText token.
    fn parse_line_comment_text(&mut self, text: &str)
        -> (AtmaToken, Pos)
    {
        if let Some(first) = text.split(&*self.newline_token).next() {
            let span = Pos {
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
        -> Option<(AtmaToken, Pos)>
    {
        let rest = text.trim_start_matches(char::is_whitespace);
        if rest.len() < text.len() {
            let substr_len = text.len() - rest.len();
            let span = Pos::new_from_string(
                &text[0..substr_len],
                &*self.newline_token);
            Some((AtmaToken::Whitespace, span))
        } else {
            None
        }
    }
}

impl Scanner for AtmaScriptScannerr {
    type Token = AtmaToken;
    type Error = TokenError;

    fn lex_prefix_token<'text>(&mut self, text: &'text str)
        -> Result<(Self::Token, Pos), (Self::Error, Pos)>
    {
        use AtmaToken::*;
        match self.open.take() {
            Some(LineCommentOpen) => {
                Ok(self.parse_line_comment_text(text))
            },

            Some(RawStringText) => {
                // Because it is necessary to recognize the RawStringClose to
                // finish parsing RawStringText, we should never get here unless
                // we know the next part of the text is the appropriately sized
                // RawStringClose token.
                let byte: usize = (self.raw_string_bracket_count + 1)
                    .try_into()
                    .expect("Pos overflow");
                Ok((RawStringClose, Pos::new(byte, 0, byte)))
            },
            Some(RawStringOpen) => {
                if let Some(parse) = self.parse_raw_string_close(text) {
                    self.raw_string_bracket_count = 0;
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_raw_string_text(text) {
                    Ok(parse)
                } else {
                    Err((TokenError("Non-terminated raw string"), Pos::ZERO))
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
                    return Ok(parse);
                }
                Err((TokenError("Unrecognized string escape"),
                    Pos::new(1, 0, 1)))
            },
            Some(StringOpenDouble) => {
                if let Some(parse) = self.parse_string_close_double(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self
                    .parse_string_text(text, StringOpenDouble)
                {
                    self.open = Some(StringOpenDouble);
                    return Ok(parse);
                }
                Err((TokenError("Unrecognized string escape"),
                    Pos::new(1, 0, 1)))
            },

            None => {
                if let Some(parse) = self.parse_command_terminator(text) {
                    return Ok(parse);
                }
                if let Some(parse) = self.parse_whitespace(text) {
                    return Ok(parse);
                }
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
                if let Some(parse) = self.parse_command_chunk(text) {
                    return Ok(parse);
                }

                Err((TokenError("Unrecognized token"), Pos::new(1, 0, 1)))
            },

            Some(_) => panic!("invalid lexer state"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Lexer tests.
////////////////////////////////////////////////////////////////////////////////


/// Tests `Lexer::new` for the AtmaScriptScannerr.
#[test]
fn as_lexer_empty() {
    let text = "";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text);

    assert_eq!(
        lexer.next(),
        None);
}


/// Tests AtmaScriptScannerr with a line comment.
#[test]
fn as_lexer_line_comment() {
    use AtmaToken::*;
    let text = "#abc\n";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (LineCommentOpen, "\"#\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentText, "\"abc\" (0:1-0:4, bytes 1-4)".to_owned()),
        (Whitespace,      "\"\n\" (0:4-1:0, bytes 4-5)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a line comment surrounded by whitespace.
#[test]
fn as_lexer_line_comment_circumfix_whitespace() {
    use AtmaToken::*;
    let text = "\n\t \n#abc\n \t\n ";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (Whitespace,      "\"\n\t \n\" (0:0-2:0, bytes 0-4)".to_owned()),
        (LineCommentOpen, "\"#\" (2:0-2:1, bytes 4-5)".to_owned()),
        (LineCommentText, "\"abc\" (2:1-2:4, bytes 5-8)".to_owned()),
        (Whitespace,      "\"\n \t\n \" (2:4-4:1, bytes 8-13)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a line comment surrounded by whitespace.
#[test]
fn as_lexer_line_comments_remove_whitespace() {
    use AtmaToken::*;
    let text = "\n\t \n#abc\n \t#def\n ";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text);
    lexer.set_filter(|tok| *tok != Whitespace);

    assert_eq!(
        lexer
            .map(|res| {
                let lex = res.unwrap();
                (*lex.token(), format!("{}", lex.span()))
            })
            .collect::<Vec<_>>(),
        vec![
            (LineCommentOpen, "\"#\" (2:0-2:1, bytes 4-5)".to_owned()),
            (LineCommentText, "\"abc\" (2:1-2:4, bytes 5-8)".to_owned()),
            (LineCommentOpen, "\"#\" (3:2-3:3, bytes 11-12)".to_owned()),
            (LineCommentText, "\"def\" (3:3-3:6, bytes 12-15)".to_owned()),
        ]);
}

/// Tests AtmaScriptScannerr with an empty single-quoted string.
#[test]
fn as_lexer_string_single_empty() {
    use AtmaToken::*;
    let text = "''";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringCloseSingle, "\"'\" (0:1-0:2, bytes 1-2)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with an unclosed single-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_single_unclosed() {
    use AtmaToken::*;
    let text = "'abc";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((StringText,  "\"abc\" (0:1-0:4, bytes 1-4)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaScriptScannerr with a non-empty single-quoted string.
#[test]
fn as_lexer_string_single_text() {
    use AtmaToken::*;
    let text = "'abc \n xyz'";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
        (StringCloseSingle, "\"'\" (1:4-1:5, bytes 10-11)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a quote-containing single-quoted string.
#[test]
fn as_lexer_string_single_quotes() {
    use AtmaToken::*;
    let text = "'abc\"\n\\'xyz'";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenSingle,  "\"'\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc\"\n\\'xyz\" (0:1-1:5, bytes 1-11)".to_owned()),
        (StringCloseSingle, "\"'\" (1:5-1:6, bytes 11-12)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaScriptScannerr with an empty double-quoted string.
#[test]
fn as_lexer_string_double_empty() {
    use AtmaToken::*;
    let text = "\"\"";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringCloseDouble, "\"\"\" (0:1-0:2, bytes 1-2)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaScriptScannerr with an unclosed double-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_double_unclosed() {
    use AtmaToken::*;
    let text = "\"abc";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((StringText,  "\"abc\" (0:1-0:4, bytes 1-4)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}


/// Tests AtmaScriptScannerr with a non-empty double-quoted string.
#[test]
fn as_lexer_string_double_text() {
    use AtmaToken::*;
    let text = "\"abc \n xyz\"";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc \n xyz\" (0:1-1:4, bytes 1-10)".to_owned()),
        (StringCloseDouble, "\"\"\" (1:4-1:5, bytes 10-11)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a quote-containing double-quoted string.
#[test]
fn as_lexer_string_double_quotes() {
    use AtmaToken::*;
    let text = "\"abc\\\"\n'xyz\"";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (0:0-0:1, bytes 0-1)".to_owned()),
        (StringText,        "\"abc\\\"\n'xyz\" (0:1-1:4, bytes 1-11)".to_owned()),
        (StringCloseDouble, "\"\"\" (1:4-1:5, bytes 11-12)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaScriptScannerr with an empty raw-quoted string.
#[test]
fn as_lexer_string_raw_empty() {
    use AtmaToken::*;
    let text = "r\"\"";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r\"\" (0:0-0:2, bytes 0-2)".to_owned()),
        (RawStringClose, "\"\"\" (0:2-0:3, bytes 2-3)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaScriptScannerr with an empty raw-quoted string using hashes.
#[test]
fn as_lexer_string_raw_empty_hashed() {
    use AtmaToken::*;
    let text = "r##\"\"##";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r##\"\" (0:0-0:4, bytes 0-4)".to_owned()),
        (RawStringClose, "\"\"##\" (0:4-0:7, bytes 4-7)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with an unclosed raw-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_raw_unclosed() {
    use AtmaToken::*;
    let text = "r###\"abc";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((RawStringOpen,  "\"r###\"\" (0:0-0:5, bytes 0-5)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((RawStringText,  "\"abc\" (0:5-0:8, bytes 5-8)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaScriptScannerr with an mismatched raw-quoted string.
#[test]
#[should_panic]
fn as_lexer_string_raw_mismatched() {
    use AtmaToken::*;
    let text = "r###\"abc\"#";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text)
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        });

    assert_eq!(
        lexer.next(), 
        Some((RawStringOpen,  "\"r###\"\" (0:0-0:5, bytes 0-5)".to_owned())));

    assert_eq!(
        lexer.next(), 
        Some((RawStringText,  "\"abc\" (0:5-0:8, bytes 5-8)".to_owned())));

    let _ = lexer.next(); // Panics due to unwrap of error.
}

/// Tests AtmaScriptScannerr with a non-empty raw-quoted string.
#[test]
fn as_lexer_string_raw_text() {
    use AtmaToken::*;
    let text = "r########\"abc \n xyz\"########";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r########\"\" (0:0-0:10, bytes 0-10)".to_owned()),
        (RawStringText,  "\"abc \n xyz\" (0:10-1:4, bytes 10-19)".to_owned()),
        (RawStringClose, "\"\"########\" (1:4-1:13, bytes 19-28)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a non-empty raw-quoted string with quotes
/// inside.
#[test]
fn as_lexer_string_raw_quoted_text() {
    use AtmaToken::*;
    let text = "r########\"abc \n xyz\"########";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (RawStringOpen,  "\"r########\"\" (0:0-0:10, bytes 0-10)".to_owned()),
        (RawStringText,  "\"abc \n xyz\" (0:10-1:4, bytes 10-19)".to_owned()),
        (RawStringClose, "\"\"########\" (1:4-1:13, bytes 19-28)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a CommandChunk.
#[test]
fn as_lexer_command_chunk() {
    use AtmaToken::*;
    let text = "abc-def";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (CommandChunk,  "\"abc-def\" (0:0-0:7, bytes 0-7)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}


/// Tests AtmaScriptScannerr with a combination of tokens.
#[test]
fn as_lexer_combined() {
    use AtmaToken::*;
    let text = "# \n\n \"abc\\\"\"'def' r##\"\t\"##\n\n\n--zyx--wvut";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);


    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (LineCommentOpen,   "\"#\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentText,   "\" \" (0:1-0:2, bytes 1-2)".to_owned()),
        (Whitespace,        "\"\n\n \" (0:2-2:1, bytes 2-5)".to_owned()),
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 5-6)".to_owned()),
        (StringText,        "\"abc\\\"\" (2:2-2:7, bytes 6-11)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:7-2:8, bytes 11-12)".to_owned()),
        (StringOpenSingle,  "\"'\" (2:8-2:9, bytes 12-13)".to_owned()),
        (StringText,        "\"def\" (2:9-2:12, bytes 13-16)".to_owned()),
        (StringCloseSingle, "\"'\" (2:12-2:13, bytes 16-17)".to_owned()),
        (Whitespace,        "\" \" (2:13-2:14, bytes 17-18)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:14-2:18, bytes 18-22)".to_owned()),
        (RawStringText,     "\"\t\" (2:18-2:19, bytes 22-23)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:19-2:22, bytes 23-26)".to_owned()),
        (Whitespace,        "\"\n\n\n\" (2:22-5:0, bytes 26-29)".to_owned()),
        (CommandChunk,      "\"--zyx--wvut\" (5:0-5:11, bytes 29-40)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a combination of tokens separated by
/// terminators.
#[test]
fn as_lexer_combined_terminated() {
    use AtmaToken::*;
    let text = ";#; \n\n \"a;bc\\\"\";'d;ef' r##\"\t;\"##;\n\n;\n--zyx;--wvut";
    let as_tok = AtmaScriptScannerr::new();
    let lexer = Lexer::new(as_tok, text);

    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (CommandTerminator, "\";\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentOpen,   "\"#\" (0:1-0:2, bytes 1-2)".to_owned()),
        (LineCommentText,   "\"; \" (0:2-0:4, bytes 2-4)".to_owned()),
        (Whitespace,        "\"\n\n \" (0:4-2:1, bytes 4-7)".to_owned()),
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 7-8)".to_owned()),
        (StringText,        "\"a;bc\\\"\" (2:2-2:8, bytes 8-14)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:8-2:9, bytes 14-15)".to_owned()),
        (CommandTerminator, "\";\" (2:9-2:10, bytes 15-16)".to_owned()),
        (StringOpenSingle,  "\"\'\" (2:10-2:11, bytes 16-17)".to_owned()),
        (StringText,        "\"d;ef\" (2:11-2:15, bytes 17-21)".to_owned()),
        (StringCloseSingle, "\"\'\" (2:15-2:16, bytes 21-22)".to_owned()),
        (Whitespace,        "\" \" (2:16-2:17, bytes 22-23)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:17-2:21, bytes 23-27)".to_owned()),
        (RawStringText,     "\"\t;\" (2:21-2:23, bytes 27-29)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:23-2:26, bytes 29-32)".to_owned()),
        (CommandTerminator, "\";\" (2:26-2:27, bytes 32-33)".to_owned()),
        (Whitespace,        "\"\n\n\" (2:27-4:0, bytes 33-35)".to_owned()),
        (CommandTerminator, "\";\" (4:0-4:1, bytes 35-36)".to_owned()),
        (Whitespace,        "\"\n\" (4:1-5:0, bytes 36-37)".to_owned()),
        (CommandChunk,      "\"--zyx\" (5:0-5:5, bytes 37-42)".to_owned()),
        (CommandTerminator, "\";\" (5:5-5:6, bytes 42-43)".to_owned()),
        (CommandChunk,      "\"--wvut\" (5:6-5:12, bytes 43-49)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a combination of tokens separated by
/// terminators with whitespace filtered out.
#[test]
fn as_lexer_combined_terminated_remove_whitespace() {
    use AtmaToken::*;
    let text = ";#; \n\n \"a;bc\\\"\";'d;ef' r##\"\t;\"##;\n\n;\n--zyx;--wvut";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text);
    lexer.set_filter(|tok| *tok != Whitespace);

    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (CommandTerminator, "\";\" (0:0-0:1, bytes 0-1)".to_owned()),
        (LineCommentOpen,   "\"#\" (0:1-0:2, bytes 1-2)".to_owned()),
        (LineCommentText,   "\"; \" (0:2-0:4, bytes 2-4)".to_owned()),
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 7-8)".to_owned()),
        (StringText,        "\"a;bc\\\"\" (2:2-2:8, bytes 8-14)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:8-2:9, bytes 14-15)".to_owned()),
        (CommandTerminator, "\";\" (2:9-2:10, bytes 15-16)".to_owned()),
        (StringOpenSingle,  "\"\'\" (2:10-2:11, bytes 16-17)".to_owned()),
        (StringText,        "\"d;ef\" (2:11-2:15, bytes 17-21)".to_owned()),
        (StringCloseSingle, "\"\'\" (2:15-2:16, bytes 21-22)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:17-2:21, bytes 23-27)".to_owned()),
        (RawStringText,     "\"\t;\" (2:21-2:23, bytes 27-29)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:23-2:26, bytes 29-32)".to_owned()),
        (CommandTerminator, "\";\" (2:26-2:27, bytes 32-33)".to_owned()),
        (CommandTerminator, "\";\" (4:0-4:1, bytes 35-36)".to_owned()),
        (CommandChunk,      "\"--zyx\" (5:0-5:5, bytes 37-42)".to_owned()),
        (CommandTerminator, "\";\" (5:5-5:6, bytes 42-43)".to_owned()),
        (CommandChunk,      "\"--wvut\" (5:6-5:12, bytes 43-49)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}

/// Tests AtmaScriptScannerr with a combination of tokens separated by
/// terminators with multiple tokens filtered out.
#[test]
fn as_lexer_combined_terminated_filtered() {
    use AtmaToken::*;
    let text = ";#; \n\n \"a;bc\\\"\";'d;ef' r##\"\t;\"##;\n\n;\n--zyx;--wvut";
    let as_tok = AtmaScriptScannerr::new();
    let mut lexer = Lexer::new(as_tok, text);
    lexer.set_filter(|tok|
        *tok != Whitespace &&
        *tok != LineCommentOpen &&
        *tok != LineCommentText &&
        *tok != CommandTerminator);

    let actual = lexer
        .map(|res| {
            let lex = res.unwrap();
            (*lex.token(), format!("{}", lex.span()))
        })
        .collect::<Vec<_>>();
    let expected = vec![
        (StringOpenDouble,  "\"\"\" (2:1-2:2, bytes 7-8)".to_owned()),
        (StringText,        "\"a;bc\\\"\" (2:2-2:8, bytes 8-14)".to_owned()),
        (StringCloseDouble, "\"\"\" (2:8-2:9, bytes 14-15)".to_owned()),
        (StringOpenSingle,  "\"\'\" (2:10-2:11, bytes 16-17)".to_owned()),
        (StringText,        "\"d;ef\" (2:11-2:15, bytes 17-21)".to_owned()),
        (StringCloseSingle, "\"\'\" (2:15-2:16, bytes 21-22)".to_owned()),
        (RawStringOpen,     "\"r##\"\" (2:17-2:21, bytes 23-27)".to_owned()),
        (RawStringText,     "\"\t;\" (2:21-2:23, bytes 27-29)".to_owned()),
        (RawStringClose,    "\"\"##\" (2:23-2:26, bytes 29-32)".to_owned()),
        (CommandChunk,      "\"--zyx\" (5:0-5:5, bytes 37-42)".to_owned()),
        (CommandChunk,      "\"--wvut\" (5:6-5:12, bytes 43-49)".to_owned()),
    ];
    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!("");
    }
    assert_eq!(actual, expected);
}
