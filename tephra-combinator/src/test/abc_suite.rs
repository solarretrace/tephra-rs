////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer tests.
////////////////////////////////////////////////////////////////////////////////
// NOTE: Run the following command to get tracing output:
// RUST_LOG=[test_name]=TRACE cargo test test_name -- --nocapture
#![allow(dead_code)]


// Internal library imports.
use crate::any;
use crate::both;
use crate::one;
use crate::seq;
use crate::spanned;
use crate::atomic;
use crate::text;

// External library imports.
use pretty_assertions::assert_eq;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Pos;
use tephra::Scanner;
use tephra::SourceText;
use tephra::Spanned;
use tephra::Success;
use test_log::test;



////////////////////////////////////////////////////////////////////////////////
// Token parser.
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum AbcToken {
    A,
    B,
    C,
    D,
    Ws,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Abc(Option<AbcToken>);

impl Abc {
    pub fn new() -> Self {
        Abc(None)
    }
}

impl std::fmt::Display for AbcToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AbcToken::*;
        match self {
            A   => write!(f, "'a'"),
            B   => write!(f, "'b'"),
            C   => write!(f, "'c'"),
            D   => write!(f, "'d'"),
            Ws  => write!(f, "whitespace"),
        }
    }
}

impl Scanner for Abc {
    type Token = AbcToken;

    fn scan<'text>(&mut self, source: SourceText<'text>, base: Pos)
        -> Option<(Self::Token, Pos)>
    {
        let text = &source.as_ref()[base.byte..];
        let metrics = source.column_metrics();

        if text.starts_with('a') {
            self.0 = Some(AbcToken::A);
            Some((
                AbcToken::A,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('b') {
            self.0 = Some(AbcToken::B);
            Some((
                AbcToken::B,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('c') {
            self.0 = Some(AbcToken::C);
            Some((
                AbcToken::C,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with("d") {
            self.0 = Some(AbcToken::D);
            Some((
                AbcToken::D,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))
            
        } else {
            self.0 = Some(AbcToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            if rest.len() < text.len() {
                let substr_len = text.len() - rest.len();
                let substr = &source.as_ref()[0.. base.byte + substr_len];
                Some((AbcToken::Ws, metrics.end_position(substr, base)))
            } else {
                self.0 = None;
                None
            }
        }
    }
}



////////////////////////////////////////////////////////////////////////////////
// Grammar
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pattern<'text> {
    Abc(Spanned<&'text str>),
    Bxx(Spanned<&'text str>),
    Xyc(Spanned<&'text str>),
}


fn pattern<'text>(lexer: Lexer<'text, Abc>)
    -> ParseResult<'text, Abc, Pattern<'text>>
{
    match atomic(spanned(text(abc)))
        (lexer.clone())
    {
        Ok(Success { value: Some(sp), lexer })
            => return Ok(Success { value: Pattern::Abc(sp), lexer }),

        _ => (),
    }

    match atomic(spanned(text(bxx)))
        (lexer.clone())
    {
        Ok(Success { value: Some(sp), lexer })
            => return Ok(Success { value: Pattern::Bxx(sp), lexer }),
        _ => (),
    }

    match spanned(text(xyc))
        (lexer)
    {
        Ok(Success { value: sp, lexer })
            => return Ok(Success { value: Pattern::Xyc(sp), lexer }),
        Err(e) => Err(e),
    }
}


fn abc<'text>(lexer: Lexer<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;

    seq(&[A, B, C])
        (lexer)
        .map_value(|v| (v[0], v[1], v[2]))
}

fn bxx<'text>(lexer: Lexer<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;

    let (c, succ) = one(B)
        (lexer)?
        .take_value();

    let (x1, succ) = any(&[A, B, C, D])
        (succ.lexer)?
        .take_value();

    one(x1)
        (succ.lexer)
        .map_value(|x2| (c, x1, x2))
}


fn xyc<'text>(lexer: Lexer<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;

    let ((x, y), succ) = both(
        any(&[A, B, C, D]),
        any(&[A, B, C, D]))
        (lexer)?
        .take_value();

    one(C)
        (succ.lexer)
        .map_value(|b| (x, y, b))
}



////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

/// Tests Abc token lexing & filtering.
#[test]
#[tracing::instrument]
fn abc_tokens() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "a b\nc d";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (
            lex.0,
            format!("{:?} ({})", source.clip(lex.1).as_str(), lex.1)))
        .collect::<Vec<_>>();

    let expected = vec![
        (A,   "\"a\" (0:0-0:1, bytes 0-1)".to_string()),
        (B,   "\"b\" (0:2-0:3, bytes 2-3)".to_string()),
        (C,   "\"c\" (1:0-1:1, bytes 4-5)".to_string()),
        (D,   "\"d\" (1:2-1:3, bytes 6-7)".to_string()),
    ];

    assert_eq!(actual, expected);
}

/// Parses a `Pattern::Abc`.
#[test]
#[tracing::instrument]
fn abc_pattern() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone())
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Bxx`.
#[test]
#[tracing::instrument]
fn bxx_pattern() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "baa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone())
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Bxx(Spanned {
        value: "baa",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Xyc`.
#[test]
#[tracing::instrument]
fn xyc_pattern() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "bac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone())
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Xyc(Spanned {
        value: "bac",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Ensures that a failure encountered after initial newline & whitespace
/// doesn't include that whitespace in the error message.
#[test]
#[tracing::instrument]
fn initial_newline_ws_skip() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "\n    zzz";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = pattern
        (lexer.clone())
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unrecognized token
 --> (1:0-1:7, bytes 1-8)
  | 
1 |     zzz
  |     \\ symbol not recognized
");
}
