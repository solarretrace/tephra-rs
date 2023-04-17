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
use crate::delimited_list;
use crate::delimited_list_bounded;
use crate::bracket_default;
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
use tephra::Span;
use tephra::Spanned;
use test_log::test;
use ntest::timeout;


// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;

////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn list_empty() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = delimited_list(
            pattern,
            Comma,
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(0, 0, 0));
}


/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn list_one() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = delimited_list(
            pattern,
            Comma,
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        }))];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}


/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_one() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                0, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn list_two() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc,aac ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = delimited_list(
            pattern,
            Comma,
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })), 
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_two() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                0, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })),
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn list_one_failed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "  ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = delimited_list_bounded(
            1, None,
            pattern,
            Comma,
            &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected more items in list
 --> (0:0-0:2, bytes 0-2)
  | 
0 |   
  |   \\ expected 1 item, found 0
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_zero() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, _succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                0, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .unwrap()
        .take_value();

    let actual = value;
    let expected = (vec![], 0);

    assert_eq!(actual, expected);
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_one_failed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                1, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected more items in list
 --> (0:0-0:2, bytes 0-2)
  | 
0 | []
  |  \\ expected 1 item, found 0
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_one_recovered() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[       ]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                1, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected more items in list
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [       ]
  |         \\ expected 1 item, found 0
");
}


/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_two_recovered_first() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[   ,aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                0, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        None,
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::new_enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [   ,aac]
  |      ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_two_recovered_second() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,   ]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                0, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::new_enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
            })),
            None,
        ],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [abc,   ]
  |         ^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test successful `delimited_list_bounded` combinator.
#[test]
#[tracing::instrument]
#[timeout(100)]
fn bracket_list_missing_delimiter() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = bracket_default(
            &[OpenBracket],
            delimited_list_bounded(
                0, None,
                pattern,
                Comma,
                &[CloseBracket]),
            &[CloseBracket],
            &[])
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![None], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [abc aac]
  |  ^^^^^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
