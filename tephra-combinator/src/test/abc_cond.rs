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
use crate::implies;
use crate::bracket;
use crate::delimited_list;
use crate::test::abc_scanner::Abc;
use crate::test::abc_scanner::AbcToken;
use crate::test::abc_scanner::pattern;
use crate::test::abc_scanner::Pattern;

// External library imports.
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::Lexer;
use tephra::Pos;
use tephra::SourceText;
use tephra::error::SourceError;
use tephra::Span;
use tephra::Spanned;
// use tephra::recover_before;
use test_log::test;

// Standard library imports.
// use std::rc::Rc;
// use std::sync::RwLock;

////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `implies` combinator.
#[test]
#[tracing::instrument]
fn pattern_implies() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc bdd";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

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
#[tracing::instrument]
fn pattern_implies_failed_left() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "agc cdd";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

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
#[tracing::instrument]
fn pattern_implies_failed_right() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc cdd";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);


    let actual = implies(pattern, pattern)
        (lexer, ctx)
        .map_err(|e|
            SourceError::convert::<AbcToken>(e.into_owned(), source))
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
#[tracing::instrument]
fn pattern_implies_list() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "abc[bdd, abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

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
        (Some(vec![
            Some(Pattern::Bxx(Spanned {
                value: "bdd",
                span: Span::new_enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
            })),
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::new_enclosing(Pos::new(9, 0, 9), Pos::new(12, 0, 12)),
            })),
        ]), 0)
    ));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(13, 0, 13));
}
