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
use crate::both;
use crate::bracket;
use crate::one;
use crate::raw;
use crate::spanned;
use crate::test::abc_scanner::Abc;
use crate::test::abc_scanner::AbcToken;
use crate::test::abc_scanner::pattern;
use crate::test::abc_scanner::Pattern;
use crate::unrecoverable;

// External library imports.
use pretty_assertions::assert_eq;
use tephra::Context;
use tephra::Lexer;
use tephra::Pos;
use tephra::SourceText;
use tephra::Span;
use tephra::Spanned;
use test_log::test;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;

////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test failed `bracket` combinator with error recovery, with missing brackets.
#[test]
#[tracing::instrument]
fn recover_missing() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket(
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
fn recover_unmatched_open() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("recover_unmatched");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[])
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> recover_unmatched:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator with error recovery, with an unmatched
/// close bracket.
#[test]
#[tracing::instrument]
fn recover_unmatched_closed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = " abc]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket(
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
fn recover_mismatched() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = bracket(
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
fn recover_unmatched_raw() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("recover_unmatched_raw");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = raw(bracket(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[]))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> recover_unmatched_raw:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator without error recovery, with an
/// unmatched bracket.
#[test]
#[tracing::instrument]
fn recover_unmatched_unrecoverable() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb  ";
    let source = SourceText::new(TEXT)
        .with_name("recover_unmatched_unrecoverable");
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = unrecoverable(bracket(
            &[OpenBracket],
            pattern,
            &[CloseBracket], &[]))
        (lexer.clone(), ctx)
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> recover_unmatched_unrecoverable:(0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test successful `bracket` combinator.
#[test]
#[tracing::instrument]
fn comma_bracket() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "a,b";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = spanned(bracket(
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

/// Test successful `bracket` combinator.
#[test]
#[tracing::instrument]
fn matching_both() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc][aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let ctx = Context::empty();
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(
            bracket(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                &[]),
            bracket(
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

/// Test successful `bracket` combinator.
#[test]
#[tracing::instrument]
fn matching_both_first_fail() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[a  ][aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = both(
            bracket(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                &[]),
            bracket(
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


/// Test `bracket` combinator failing due to mismatched brackets.
#[test]
#[tracing::instrument]
fn matching_both_mismatch() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[abc,[aac]";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e| errors.write().unwrap().push(e))));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let actual = both(
            bracket(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                &[]),
            bracket(
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



