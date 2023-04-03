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
use crate::unrecoverable;
use crate::bracket_matching;
use crate::recover_option;
use crate::recover_until;
use crate::seq;
use crate::spanned;
use crate::raw;
use crate::text;
use crate::maybe;
use crate::center;
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
use tephra::SpanDisplay;
use tephra::Span;
use tephra::Spanned;
use tephra::recover_before;
use tephra::Success;
use tephra::ParseError;
use test_log::test;

// Standard library imports.
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
    Invalid,
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
            Invalid      => write!(f, "invalid token"),
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
            
        } else if text.starts_with(char::is_whitespace) {
            self.0 = Some(AbcToken::Ws);
            let rest = text.trim_start_matches(char::is_whitespace);
            
            let substr_len = text.len() - rest.len();
            let substr = &source.as_ref()[0.. base.byte + substr_len];
            Some((AbcToken::Ws, metrics.end_position(substr, base)))
        } else if text.len() > 0 {
            self.0 = Some(AbcToken::Invalid);
            Some((
                AbcToken::Invalid,
                metrics.end_position(&source.as_ref()[..base.byte + 1], base)))
        } else {
            self.0 = None;
            None
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

fn pattern<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, Pattern<'text>>
{
    match maybe(spanned(text(abc)))
        (lexer.clone(), ctx.clone())
    {
        Ok(Success { value: Some(sp), lexer }) => {
            return Ok(Success { value: Pattern::Abc(sp), lexer });
        },
        _ => (),
    }

    match maybe(spanned(text(bxx)))
        (lexer.clone(), ctx.clone())
    {
        Ok(Success { value: Some(sp), lexer }) => {
            return Ok(Success { value: Pattern::Bxx(sp), lexer });
        },
        _ => (),
    }

    // Setup error context.
    let source = lexer.source_text();
    let ctx = ctx.push(std::rc::Rc::new(move |_e, lexer| ParseError::new(
            source.clone(),
            "expected pattern")
        .with_span_display(SpanDisplay::new_error_highlight(
            source,
            lexer.parse_span(),
            "expected 'ABC', 'BXX', or 'XYC' pattern"))));

    spanned(text(xyc))
        (lexer, ctx.clone())
        .apply_context(ctx)
        .map_value(Pattern::Xyc)
}

fn abc<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;
    seq(&[A, B, C])
        (lexer, ctx)
        .map_value(|v| (v[0], v[1], v[2]))
}

fn bxx<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
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

fn xyc<'text>(lexer: Lexer<'text, Abc>, ctx: Context<'text, Abc>)
    -> ParseResult<'text, Abc, (AbcToken, AbcToken, AbcToken)>
{
    use AbcToken::*;
    let res = both(
        any(&[A, B, C, D]),
        any(&[A, B, C, D]))
        (lexer, ctx.clone());
    let ((x, y), succ) = res
        .map_lexer_failure(|mut l| { l.advance_to_unfiltered(Ws); l })?
        .take_value();

    one(C)
        (succ.lexer, ctx)
        .map_lexer_failure(|mut l| { l.advance_to_unfiltered(Ws); l })
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
    const TEXT: &'static str = "\n    aaa";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = pattern
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected pattern
 --> (1:0-1:7, bytes 1-8)
  | 
1 |     aaa
  |     ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `pred` combinator.
#[test]
#[tracing::instrument]
fn simple_not() {
    colored::control::set_override(false);

    use crate::pred;
    use crate::Expr;
    use AbcToken::*;
    const TEXT: &'static str = "abc dac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = pred(Expr::Not(Box::new(Expr::Var(D))))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = A;

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(1, 0, 1));
}

/// Test failed `pred` combinator.
#[test]
#[tracing::instrument]
fn simple_not_failed() {
    colored::control::set_override(false);

    use crate::pred;
    use crate::Expr;
    use AbcToken::*;
    const TEXT: &'static str = "dabc dac";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = pred(Expr::Not(Box::new(Expr::Var(D))))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unexpected token
 --> (0:0-0:8, bytes 0-8)
  | 
0 | dabc dac
  | ^ expected DnfVec([Not(Var(Token(D)))])
");
}

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
error: expected pattern
 --> (0:0-0:7, bytes 0-7)
  | 
0 | abc ddd
  |     ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
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

/// Test successful `center` combinator.
#[test]
#[tracing::instrument]
fn pattern_center() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = center(
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

/// Test failed `center` combinator with error recovery.
#[test]
#[tracing::instrument]
fn pattern_center_recover() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = center(
            one(OpenBracket),
            recover_option(pattern, recover_before(CloseBracket)),
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
error: expected pattern
 --> (0:0-0:4, bytes 0-4)
  | 
0 | [ab]
  |  ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test failed `center` combinator with error recovery, with a delayed close center.
#[test]
#[tracing::instrument]
fn pattern_center_recover_delayed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb] ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = center(
            one(OpenBracket),
            recover_option(pattern, recover_before(CloseBracket)),
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
error: expected pattern
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb] 
  |  ^^^^^^^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test failed `bracket` combinator with error recovery, with missing brackets.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_missing() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_matching(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected open bracket
 --> (0:0-0:5, bytes 0-5)
  | 
0 |  abc 
  |  \\ bracket expected here
");
}


/// Test failed `bracket` combinator with error recovery, with an unmatched open
/// bracket.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_unmatched_open() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("pattern_bracket_recover_unmatched");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_matching(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> pattern_bracket_recover_unmatched:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator with error recovery, with an unmatched
/// close bracket.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_unmatched_closed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_matching(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched close bracket
 --> (0:0-0:5, bytes 0-5)
  | 
0 |  abc]
  |     ^ this bracket has no matching open
");
}

/// Test failed `bracket` combinator with error recovery, with an unmatched
/// close bracket.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_mismatched() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_matching(
            &[OpenBracket, OpenBracket],
            pattern,
            &[CloseBracket, Comma], &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched brackets
 --> (0:0-0:5, bytes 0-5)
  | 
0 | [abc,
  | ^ the bracket here
  |     ^ ... does not match the closing bracket here
");
}

/// Test failed `bracket` combinator with error recovery, with an
/// unmatched bracket and raw error.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_unmatched_raw() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("pattern_bracket_recover_unmatched_raw");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = raw(bracket_matching(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[]))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> pattern_bracket_recover_unmatched_raw:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator without error recovery, with an
/// unmatched bracket.
#[test]
#[tracing::instrument]
fn pattern_bracket_recover_unmatched_unrecoverable() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("pattern_bracket_recover_unmatched_unrecoverable");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = unrecoverable(bracket_matching(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[]))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> pattern_bracket_recover_unmatched_unrecoverable:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test successful `bracket_matching` combinator.
#[test]
#[tracing::instrument]
fn comma_bracket_matching() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "a,b";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = spanned(bracket_matching(
            &[A],
            one(Comma),
            &[B],
            &[]))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Spanned {
        value: (Some(Comma), 0),
        span: Span::new_enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
    };
    println!("{:?}", actual);
    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Test successful `bracket_matching` combinator.
#[test]
#[tracing::instrument]
fn pattern_bracket_matching_both() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc][aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(
            bracket_matching(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                &[]),
            bracket_matching(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                &[]))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = ((Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })), 0), 
        (Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
        })), 0));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));
}

/// Test successful `bracket_matching` combinator.
#[test]
#[tracing::instrument]
fn pattern_bracket_matching_both_first_fail() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[a  ][aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(
            bracket_matching(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                &[]),
            bracket_matching(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                &[]))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (
        (None, 0), 
        (Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
        })), 0));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [a  ][aac]
  |  ^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}


/// Test `bracket_matching` combinator failing due to mismatched brackets.
#[test]
#[tracing::instrument]
fn pattern_bracket_matching_both_mismatch() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,[aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = both(
            bracket_matching(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                &[]),
            bracket_matching(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                &[]))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched brackets
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [abc,[aac]
  | ^ the bracket here
  |     ^ ... does not match the closing bracket here
");
}



