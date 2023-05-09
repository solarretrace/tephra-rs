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
use crate::both;
use crate::center;
use crate::left;
use crate::one;
use crate::raw;
use crate::recover;
use crate::right;
use crate::stabilize;
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
use tephra::recover_before;
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
    let ctx_errors = errors.clone();
    let ctx = Context::new(Some(Box::new(move |e| 
        ctx_errors.write().unwrap().push(e.into_source_error(source))
    )));

    (lexer, ctx, errors, source)
}

////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `pred` combinator.
#[test]
#[timeout(100)]
fn simple_not() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("abc dac");
    use AbcToken::*;

    use crate::pred;
    use crate::Expr;

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
#[timeout(100)]
fn simple_not_failed() {
    test_setup();
    let (lexer, ctx, _errors, source) = test_parser("dabc dac");
    use AbcToken::*;

    use crate::pred;
    use crate::Expr;

    let actual = pred(Expr::Not(Box::new(Expr::Var(D))))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
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
#[timeout(100)]
fn pattern_both() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("abc dac");

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
#[timeout(100)]
fn pattern_left() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("abc dac");

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
#[timeout(100)]
fn pattern_right() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("abc dac");

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
#[timeout(100)]
fn pattern_right_failed() {
    test_setup();
    let (lexer, ctx, _errors, source) = test_parser("abc ddd");

    let actual = right(pattern, pattern)
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
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
#[timeout(100)]
fn pattern_right_failed_raw() {
    test_setup();
    let (lexer, ctx, _errors, source) = test_parser("abc ddd");

    let actual = raw(right(pattern, pattern))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
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
#[timeout(100)]
fn pattern_center() {
    test_setup();
    let (lexer, ctx, _errors, _source) = test_parser("[abc]");
    use AbcToken::*;

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
#[timeout(100)]
fn pattern_center_recover() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[ab]");
    use AbcToken::*;

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
  |  ^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}

/// Test failed `center` combinator with error recovery, with a delayed close
/// center.
#[test]
#[timeout(100)]
fn pattern_center_recover_delayed() {
    test_setup();
    let (lexer, ctx, errors, _source) = test_parser("[ab   bbb] ");
    use AbcToken::*;

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
  |  ^^^^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
