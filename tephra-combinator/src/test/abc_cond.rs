////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Lexer tests.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::bracket;
use crate::delimited_list;
use crate::implies;
use crate::test::abc_scanner::Abc;
use crate::test::abc_scanner::AbcToken;
use crate::test::abc_scanner::pattern;
use crate::test::abc_scanner::Pattern;

// External library imports.
use ntest::timeout;
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::error::SourceError;
use tephra::Lexer;
use tephra::Pos;
use tephra::SourceText;
use tephra::Span;
use tephra::Spanned;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;


////////////////////////////////////////////////////////////////////////////////
// Test setup
////////////////////////////////////////////////////////////////////////////////
fn test_setup() {
    colored::control::set_override(false);
}

fn test_parser(text: &'static str) -> (
    Lexer<'static, Abc>,
    Context<'static, Abc>,
    Rc<RwLock<Vec<SourceError<&'static str>>>>,
    SourceText<&'static str>)
{
    let source = SourceText::new(text);
    let mut lexer = Lexer::new(Abc::new(), source);
    lexer.set_filter_fn(|tok| *tok != AbcToken::Ws);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::empty();

    (lexer, ctx, errors, source)
}


////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `implies` combinator.
#[test]
#[timeout(100)]
fn pattern_implies() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("abc bdd");

    let (value, succ) = implies(pattern, pattern)
        (lexer, ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Some((
        Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
        }),
        Pattern::Bxx(Spanned {
            value: "bdd",
            span: Span::new_enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
        }),
    ));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}

/// Test successful `implies` combinator.
#[test]
#[timeout(100)]
fn pattern_implies_failed_left() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("agc cdd");

    let (value, succ) = implies(pattern, pattern)
        (lexer, ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = None;

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(0, 0, 0));
}


/// Test successful `implies` combinator.
#[test]
#[timeout(100)]
fn pattern_implies_failed_right() {
    test_setup();
    let (lexer, ctx, _errors, source) = test_parser("abc cdd");

    let actual = implies(pattern, pattern)
        (lexer, ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected pattern
 --> (0:0-0:7, bytes 0-7)
  | 
0 | abc cdd
  |     ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test successful `implies` combinator.
#[test]
#[timeout(100)]
fn pattern_implies_list() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("abc[bdd, abc]");
    use AbcToken::*;

    let (value, succ) = implies(
            pattern,
            bracket(
                &[OpenBracket],
                delimited_list(
                    pattern,
                    Comma,
                    |tok| *tok == CloseBracket),
                &[CloseBracket],
                |_| false))
        (lexer, ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Some((
        Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
        }),
        Some(vec![
            Some(Pattern::Bxx(Spanned {
                value: "bdd",
                span: Span::new_enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
            })),
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::new_enclosing(Pos::new(9, 0, 9), Pos::new(12, 0, 12)),
            })),
        ]),
    ));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(13, 0, 13));
}
