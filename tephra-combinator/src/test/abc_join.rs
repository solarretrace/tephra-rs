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
use crate::sub;
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
use tephra_tracing::Level;
use tephra_tracing::span;
use tracing::subscriber::DefaultGuard;
use tracing::subscriber::set_default;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::Registry;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;


////////////////////////////////////////////////////////////////////////////////
// Test setup
////////////////////////////////////////////////////////////////////////////////
fn setup_test_environment() -> DefaultGuard {
    // Disable colored output for SourceError display output.
    colored::control::set_override(false);

    let env_filter_layer = EnvFilter::from_default_env();
    let fmt_layer = Layer::new().without_time().with_ansi(false);

    let subscriber = Registry::default()
        .with(env_filter_layer)
        .with(fmt_layer);

    set_default(subscriber)
}

fn build_test_lexer(text: &'static str) -> (
    Lexer<'static, Abc>,
    Context<'static, Abc>,
    Rc<RwLock<Vec<SourceError<&'static str>>>>,
    SourceText<&'static str>)
{
    let source = SourceText::new(text);
    let lexer = Lexer::new(Abc::new(), source)
        .with_filter(Some(Rc::new(|tok| *tok != AbcToken::Ws)));
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::simple_not -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn simple_not() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "simple_not")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("abc dac");
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::simple_not_failed -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn simple_not_failed() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "simple_not_failed")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("dabc dac");
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_both -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_both() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_both")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("abc dac");

    let (value, succ) = both(pattern, pattern)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (
        Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
        }),
        Pattern::Xyc(Spanned {
            value: "dac",
            span: Span::enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
        }),
    );

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}


/// Test successful `left` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_left -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_left() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_left")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("abc dac");

    let (value, succ) = left(pattern, pattern)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: Span::enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}


/// Test successful `right` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_right -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_right() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_right")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("abc dac");

    let (value, succ) = right(pattern, pattern)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Pattern::Xyc(Spanned {
        value: "dac",
        span: Span::enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}


/// Test `right` combinator failure. Ensure error is properly wrapped.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_right_failed -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_right_failed() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_right_failed")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("abc ddd");

    let actual = right(pattern, sub(pattern))
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_right_failed_raw -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_right_failed_raw() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_right_failed_raw")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("abc ddd");

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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_center -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_center() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_center")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[abc]");
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
        span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}


/// Test failed `center` combinator with error recovery.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_center_recover -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_center_recover() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_center_recover")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[ab]");
    use AbcToken::*;

    let (value, succ) = center(
            one(OpenBracket),
            recover(sub(pattern), recover_before(CloseBracket)),
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_join::pattern_center_recover_delayed -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_center_recover_delayed() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_center_recover_delayed")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[ab   bbb] ");
    use AbcToken::*;

    let (value, succ) = center(
            one(OpenBracket),
            recover(sub(pattern), recover_before(CloseBracket)),
            stabilize(one(CloseBracket)))
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = None;

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: expected pattern
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb] 
  |  ^^^^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
