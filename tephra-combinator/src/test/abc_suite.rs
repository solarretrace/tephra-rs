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
use crate::recover_option;
use crate::recover_until;
use crate::seq;
use crate::spanned;
use crate::raw;
use crate::text;
use crate::maybe;
use crate::bracket;
use crate::left;
use crate::right;

// External library imports.
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::Lexer;
use tephra::ParseResult;
use tephra::ParseResultExt as _;
use tephra::Pos;
use tephra::Scanner;
use tephra::SourceText;
use tephra::Span;
use tephra::Spanned;
use tephra::Recover;
use tephra::Success;
use tephra::ParseError;
use test_log::test;

use std::rc::Rc;
use std::sync::RwLock;


////////////////////////////////////////////////////////////////////////////////
// Abc scanner
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum AbcToken {
    A,
    B,
    C,
    D,
    Ws,
    Comma,
    Semicolon,
    OpenBracket,
    CloseBracket,
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
            A            => write!(f, "'a'"),
            B            => write!(f, "'b'"),
            C            => write!(f, "'c'"),
            D            => write!(f, "'d'"),
            Ws           => write!(f, "whitespace"),
            Comma        => write!(f, "','"),
            Semicolon    => write!(f, "','"),
            OpenBracket  => write!(f, "'['"),
            CloseBracket => write!(f, "']'"),
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

        if text.starts_with(',') {
            self.0 = Some(AbcToken::Comma);
            Some((
                AbcToken::Comma,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with(';') {
            self.0 = Some(AbcToken::Semicolon);
            Some((
                AbcToken::Semicolon,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with(']') {
            self.0 = Some(AbcToken::CloseBracket);
            Some((
                AbcToken::CloseBracket,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('[') {
            self.0 = Some(AbcToken::OpenBracket);
            Some((
                AbcToken::OpenBracket,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))

        } else if text.starts_with('a') {
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
// Abc grammar
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pattern<'text> {
    Abc(Spanned<&'text str>),
    Bxx(Spanned<&'text str>),
    Xyc(Spanned<&'text str>),
}


fn pattern<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text>)
    -> ParseResult<'text, Abc, Pattern<'text>>
{
    match maybe(spanned(text(abc)))
        (lexer.clone(), ctx.clone())
    {
        Ok(Success { value: Some(sp), lexer }) => {
            return Ok(Success { value: Pattern::Abc(sp), lexer });
        },
        _        => (),
    }

    match maybe(spanned(text(bxx)))
        (lexer.clone(), ctx.clone())
    {
        Ok(Success { value: Some(sp), lexer }) => {
            return Ok(Success { value: Pattern::Bxx(sp), lexer });
        },
        _        => (),
    }

    // Setup error context.
    let source = lexer.source();
    let ctx = ctx.push(std::rc::Rc::new(move |e| e
        .with_error_context(ParseError::new(source, "unrecognized pattern"))));

    spanned(text(xyc))
        (lexer, ctx.clone())
        .apply_context(ctx)
        .map_value(Pattern::Xyc)
}


fn abc<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;
    seq(&[A, B, C])
        (lexer, ctx)
        .map_value(|v| (v[0], v[1], v[2]))
}

fn bxx<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;

    let (c, succ) = one(B)
        (lexer, ctx.clone())?
        .take_value();

    let (x1, succ) = any(&[A, B, C, D])
        (succ.lexer, ctx.clone())?
        .take_value();

    one(x1)
        (succ.lexer, ctx)
        .map_value(|x2| (c, x1, x2))
}


fn xyc<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;
    let ((x, y), succ) = both(
        any(&[A, B, C, D]),
        any(&[A, B, C, D]))
        (lexer, ctx.clone())?
        .take_value();

    one(C)
        (succ.lexer, ctx)
        .map_value(|b| (x, y, b))
}



////////////////////////////////////////////////////////////////////////////////
// Misc tests
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
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone(), ctx)
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
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone(), ctx)
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
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pattern
        (lexer.clone(), ctx)
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
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = pattern
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unrecognized pattern
... caused by error: unrecognized token
 --> (1:0-1:7, bytes 1-8)
  | 
1 |     zzz
  |     \\ symbol not recognized
");
}


////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `both` combinator.
#[test]
#[tracing::instrument]
fn pattern_both() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc dac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(pattern, pattern)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = (
        Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
        }),
        Pattern::Xyc(Spanned {
            value: "dac",
            span: Span::new_enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
        }),
    );

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}


/// Test successful `left` combinator.
#[test]
#[tracing::instrument]
fn pattern_left() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc dac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = left(pattern, pattern)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: Span::new_enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}



/// Test successful `right` combinator.
#[test]
#[tracing::instrument]
fn pattern_right() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc dac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = right(pattern, pattern)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Pattern::Xyc(Spanned {
        value: "dac",
        span: Span::new_enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}


/// Test `right` combinator failure. Ensure error is properly wrapped.
#[test]
#[tracing::instrument]
fn pattern_right_failed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc ddd";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);


    let actual = right(pattern, pattern)
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unrecognized pattern
... caused by error: unexpected token
 --> (0:0-0:7, bytes 0-7)
  | 
0 | abc ddd
  |       ^ expected 'c'
");
}

/// Test failed `right` combinator with `raw` wrapper. Ensure error is not
/// wrapped.
#[test]
#[tracing::instrument]
fn pattern_right_failed_raw() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc ddd";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);


    let actual = raw(right(pattern, pattern))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unexpected token
 --> (0:0-0:7, bytes 0-7)
  | 
0 | abc ddd
  |       ^ expected 'c'
");
}


/// Test successful `bracket` combinator.
#[test]
#[tracing::instrument]
fn pattern_bracket() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket(
            one(OpenBracket),
            pattern,
            one(CloseBracket))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}


/// Test failed `bracket` combinator with error recovery.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket(
            one(OpenBracket),
            recover_option(pattern, Recover::before(CloseBracket)),
            recover_until(one(CloseBracket)))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = None;

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(4, 0, 4));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: unrecognized pattern
... caused by error: unexpected token
 --> (0:0-0:4, bytes 0-4)
  | 
0 | [ab]
  |    ^ expected 'c'
");
}


/// Test failed `bracket` combinator with error recovery, with a delayed close bracket.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_delayed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb] ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket(
            one(OpenBracket),
            recover_option(pattern, Recover::before(CloseBracket)),
            recover_until(one(CloseBracket)))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = None;

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(11, 0, 11));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: unrecognized pattern
... caused by error: unexpected token
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb] 
  |       ^ expected 'c'
");
}

/// Test failed `bracket` combinator with error recovery, with a delayed close bracket.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_unpaired() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket(
            one(OpenBracket),
            recover_option(pattern, Recover::before(CloseBracket)),
            recover_until(one(CloseBracket)))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = None;

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(11, 0, 11));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: unrecognized pattern
... caused by error: unexpected token
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  |       ^ expected 'c'
");
}
