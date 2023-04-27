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
use crate::center;
use crate::left;
use crate::one;
use crate::raw;
use crate::recover;
use crate::stabilize;
use crate::right;
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
use tephra::common::SourceError;
use tephra::Span;
use tephra::Spanned;
use tephra::recover_before;
use test_log::test;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;

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
        .map_err(|e|
            SourceError::convert::<AbcToken>(e.into_owned(), source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unexpected token
 --> (0:0-0:8, bytes 0-8)
  | 
0 | dabc dac
  | ^ expected DnfVec([Not(Var(Token(D)))]); found 'd'
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
        .map_err(|e|
            SourceError::convert::<AbcToken>(e.into_owned(), source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected pattern
 --> (0:0-0:7, bytes 0-7)
  | 
0 | abc ddd
  | ^^^^^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
    // NOTE: This error message is odd, but it is due to a poorly written parser
    // composition. Since there is no sublex between the patterns, the whole
    // parse fails or succeeds together.
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
        .map_err(|e|
            SourceError::convert::<AbcToken>(e.into_owned(), source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unexpected token
 --> (0:0-0:7, bytes 0-7)
  | 
0 | abc ddd
  |       ^ expected 'c'; found 'd'
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
    let ctx = Context::new(Some(Box::new(|e| 
        errors.write().unwrap().push(SourceError::convert::<AbcToken>(e.into_owned(), source))
    )));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = center(
            one(OpenBracket),
            recover(pattern, recover_before(CloseBracket)),
            stabilize(one(CloseBracket)))
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
  | ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
    // NOTE: This error message is odd, but it is due to a poorly written parser
    // composition. The error context for this parse should know that there's a
    // bracket prefix and a better error message should be written. (Or a
    // bracket combinator should be used.)
}

/// Test failed `center` combinator with error recovery, with a delayed close
/// center.
#[test]
#[tracing::instrument]
fn pattern_center_recover_delayed() {
    colored::control::set_override(false);

    use AbcToken::*;
    const TEXT: &'static str = "[ab   bbb] ";
    let source = SourceText::new(TEXT);
    let mut lexer = Lexer::new(Abc::new(), source);
    let errors = Rc::new(RwLock::new(Vec::new()));
    let ctx = Context::new(Some(Box::new(|e|
        errors.write().unwrap()
            .push(SourceError::convert::<AbcToken>(e.into_owned(), source)))
    ));
    lexer.set_filter_fn(|tok| *tok != Ws);

    let (value, succ) = center(
            one(OpenBracket),
            recover(pattern, recover_before(CloseBracket)),
            stabilize(one(CloseBracket)))
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
  | ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
    // NOTE: This error message is odd, but it is due to a poorly written parser
    // composition. The error context for this parse should know that there's a
    // bracket prefix and a better error message should be written. (Or a
    // bracket combinator should be used.)
}
