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
use crate::bracket;
use crate::bracket_index;
use crate::one;
use crate::raw;
use crate::spanned;
use crate::test::abc_scanner::Abc;
use crate::test::abc_scanner::AbcToken;
use crate::test::abc_scanner::pattern;
use crate::test::abc_scanner::Pattern;
use crate::unrecoverable;

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
/// Test failed `bracket` combinator with error recovery, with missing brackets.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::recover_missing -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn recover_missing() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "recover_missing")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer(" abc ");
    use AbcToken::*;

    let actual = bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false)
        (lexer, ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected open bracket
 --> (0:0-0:5, bytes 0-5)
  | 
0 |  abc 
  | \\ bracket expected here
");
}


/// Test failed `bracket` combinator with error recovery, with an unmatched open
/// bracket.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::recover_unmatched_open -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn recover_unmatched_open() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "recover_unmatched_open")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("[ab   bbb  ");
    use AbcToken::*;


    let actual = bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e|
            e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator with error recovery, with an unmatched
/// close bracket.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::recover_unmatched_closed -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn recover_unmatched_closed() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "recover_unmatched_closed")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer(" abc]");
    use AbcToken::*;

    let actual = bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::recover_mismatched -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn recover_mismatched() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "recover_mismatched")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("[abc,");
    use AbcToken::*;

    let actual = bracket_index(
            &[OpenBracket, OpenBracket],
            pattern,
            &[CloseBracket, Comma], |_| false)
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: mismatched brackets
 --> (0:0-0:5, bytes 0-5)
  | 
0 | [abc,
  | ^ the bracket here
  |     ^ ... does not match the closing bracket here
");
}

/// Test failed `bracket` combinator with error recovery, with an
/// unmatched bracket and raw error.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::recover_unmatched_raw -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn recover_unmatched_raw() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "recover_unmatched_raw")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("[ab   bbb  ");
    use AbcToken::*;

    let actual = raw(bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}

/// Test failed `bracket` combinator without error recovery, with an
/// unmatched bracket.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::recover_unmatched_unrecoverable -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn recover_unmatched_unrecoverable() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "recover_unmatched_unrecoverable")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("[ab   bbb  ");
    use AbcToken::*;

    let actual = unrecoverable(bracket_index(
            &[OpenBracket],
            pattern,
            &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: unmatched open bracket
 --> (0:0-0:11, bytes 0-11)
  | 
0 | [ab   bbb  
  | ^ this bracket is not closed
");
}
/// Test successful `bracket` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::comma_bracket_index -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn comma_bracket_index() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "comma_bracket_index")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("a,b");
    use AbcToken::*;

    let (value, succ) = spanned(bracket_index(
            &[A],
            one(Comma),
            &[B],
            |_| false))
        (lexer.clone(), ctx)
        .expect("perform successful parse")
        .take_value();

    let actual = value;
    let expected = Spanned {
        value: (Some(Comma), 0),
        span: Span::enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
    };
    println!("{actual:?}");
    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}
/// Test successful `bracket` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::matching_both -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn matching_both() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "matching_both")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[abc][aac]");
    use AbcToken::*;

    let (value, succ) = both(
            bracket_index(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                |_| false),
            bracket_index(
                &[OpenBracket],
                pattern,
                &[CloseBracket],
                |_| false))
        (lexer.clone(), ctx)
        .expect("perform successful parse")
        .take_value();

    let actual = value;
    let expected = ((Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })), 0), 
        (Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
        })), 0));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));
}

/// Test successful `bracket` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::matching_both_first_fail -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn matching_both_first_fail() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "matching_both_first_fail")
        .entered();
    let (lexer, ctx, errors, _) = build_test_lexer("[a  ][aac]");
    use AbcToken::*;

    let (value, succ) = both(
            bracket(
                &[OpenBracket],
                pattern,
                &[CloseBracket], |_| false),
            bracket(
                &[OpenBracket],
                pattern,
                &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .expect("perform successful parse")
        .take_value();

    let actual = value;
    let expected = (
        None, 
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
        })));

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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_bracket::matching_both_mismatch -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn matching_both_mismatch() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "matching_both_mismatch")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("[abc,[aac]");
    use AbcToken::*;

    let actual = both(
            bracket_index(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                |_| false),
            bracket_index(
                &[OpenBracket, OpenBracket],
                pattern,
                &[CloseBracket, Comma],
                |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: mismatched brackets
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [abc,[aac]
  | ^ the bracket here
  |     ^ ... does not match the closing bracket here
");
}

