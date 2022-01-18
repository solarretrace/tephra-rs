////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer tests.
////////////////////////////////////////////////////////////////////////////////
#![allow(dead_code)]

// Local imports.
use crate::combinator::both;
use crate::combinator::one;
use crate::combinator::text;
use crate::combinator::any;
use crate::combinator::seq;
use crate::combinator::spanned;
use crate::combinator::atomic;
use crate::lexer::Lexer;
use crate::lexer::Scanner;
use crate::position::ColumnMetrics;
use crate::position::Pos;
use crate::span::Span;
use crate::result::ParseResultExt as _;
use crate::result::ParseResult;
use crate::result::Spanned;
use crate::result::Success;


// External library imports.
use pretty_assertions::assert_eq;



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

    fn scan<'text>(
        &mut self,
        source: &'text str,
        base: Pos,
        metrics: ColumnMetrics)
        -> Option<(Self::Token, Pos)>
    {
        let text = &source[base.byte..];

        if text.starts_with('a') {
            self.0 = Some(AbcToken::A);
            Some((
                AbcToken::A,
                metrics.end_position(&source[..base.byte + 1], base)))

        } else if text.starts_with('b') {
            self.0 = Some(AbcToken::B);
            Some((
                AbcToken::B,
                metrics.end_position(&source[..base.byte + 1], base)))

        } else if text.starts_with('c') {
            self.0 = Some(AbcToken::C);
            Some((
                AbcToken::C,
                metrics.end_position(&source[..base.byte + 1], base)))

        } else if text.starts_with("d") {
            self.0 = Some(AbcToken::D);
            Some((
                AbcToken::D,
                metrics.end_position(&source[..base.byte + 1], base)))
            
        } else {
            self.0 = Some(AbcToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            if rest.len() < text.len() {
                let substr_len = text.len() - rest.len();
                let substr = &source[0.. base.byte + substr_len];
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
    Abc(Spanned<'text, &'text str>),
    Bxx(Spanned<'text, &'text str>),
    Xyc(Spanned<'text, &'text str>),
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
fn abc_tokens() {
    use AbcToken::*;
    const TEXT: &'static str = "a b\nc d";
    let mut lexer = Lexer::new(Abc::new(), TEXT);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (lex.0, format!("{:?}", lex.1)))
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
fn abc_pattern() {
    use AbcToken::*;
    const TEXT: &'static str = "abc";
    let mut lexer = Lexer::new(Abc::new(), TEXT);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone())
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: Span::full(TEXT, ColumnMetrics::default()),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Bxx`.
#[test]
fn bxx_pattern() {
    use AbcToken::*;
    const TEXT: &'static str = "baa";
    let mut lexer = Lexer::new(Abc::new(), TEXT);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone())
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Bxx(Spanned {
        value: "baa",
        span: Span::full(TEXT, ColumnMetrics::default()),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Xyc`.
#[test]
fn xyc_pattern() {
    use AbcToken::*;
    const TEXT: &'static str = "bac";
    let mut lexer = Lexer::new(Abc::new(), TEXT);
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone())
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Xyc(Spanned {
        value: "bac",
        span: Span::full(TEXT, ColumnMetrics::default()),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}
