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
use crate::bracket_default_index;
use crate::list;
use crate::list_bounded;
use crate::test::abc::Abc;
use crate::test::abc::AbcToken;
use crate::test::abc::pattern;
use crate::test::abc::Pattern;
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

/// Test successful `list` combinator with empty list.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::list_empty -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn list_empty() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "list_empty")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("");
    use AbcToken::*;

    let (value, succ) = list(
            pattern,
            Comma, |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(0, 0, 0));
}

/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::list_one -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn list_one() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "list_one")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer(" abc ");
    use AbcToken::*;

    let (value, succ) = list(
            pattern,
            Comma, |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        }))];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(4, 0, 4));
}

/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_one -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_one() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_one")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[abc]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}

/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::list_two -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn list_two() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "list_two")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer(" abc,aac ");
    use AbcToken::*;

    let (value, succ) = list(
            pattern,
            Comma,
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = [
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })), 
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))];

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(8, 0, 8));
}

/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_two -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_two() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_two")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[abc,aac]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        Some(Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        })),
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
        }))],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));
}

/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::list_one_failed -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn list_one_failed() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "list_one_failed")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("  ");
    use AbcToken::*;

    let actual = unrecoverable(
        list_bounded(
            1, None,
            pattern,
            Comma, |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: invalid item count
 --> (0:0-0:2, bytes 0-2)
  | 
0 |   
  |   \\ expected 1 item; found 0
");
}


/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_zero -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_zero() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_zero")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[]");
    use AbcToken::*;

    let (value, _succ) = bracket_default_index(
            &[OpenBracket],
            list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .unwrap()
        .take_value();

    let actual = value;
    let expected = (vec![], 0);

    assert_eq!(actual, expected);
}


/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_one_failed -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_one_failed() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_one_failed")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("[]");
    use AbcToken::*;

    let actual = unrecoverable(
        bracket_default_index(
            &[OpenBracket],
            list_bounded(
                1, None,
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket], |_| false))
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: invalid item count
 --> (0:0-0:2, bytes 0-2)
  | 
0 | []
  |  \\ expected 1 item; found 0
");
}


/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_one_recovered -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_one_recovered() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_one_recovered")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[       ]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list_bounded(
                1, None,
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: invalid item count
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [       ]
  |         \\ expected 1 item; found 0
");
}


/// Tests parse of a bracketed list with error recovery for the first position.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_two_recovered_first -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_two_recovered_first() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_two_recovered_first")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[   ,aac]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list(
                pattern,
                Comma, |tok| *tok == CloseBracket),
            &[CloseBracket], |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
        None,
        Some(Pattern::Xyc(Spanned {
            value: "aac",
            span: Span::enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
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
  |     \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
}


/// Tests parse of a bracketed list with error recovery for the second position.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_two_recovered_second -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_two_recovered_second() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_two_recovered_second")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[abc,   ]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
            })),
        ],
        0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 0);
}


/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_missing_delimiter -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_missing_delimiter() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_missing_delimiter")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[abc aac]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list(
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![None], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(9, 0, 9));

    assert_eq!(errors.read().unwrap().len(), 1);
    assert_eq!(format!("{}", errors.write().unwrap().pop().unwrap()), "\
error: incomplete parse
 --> (0:0-0:9, bytes 0-9)
  | 
0 | [abc aac]
  |     ^^^^^ unexpected text
");
}


/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_nested -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_nested() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_nested")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[[],[abc, abc], [aac]]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list(
                bracket_default_index(
                    &[OpenBracket],
                    list(
                        pattern,
                        Comma,
                        |tok| *tok == CloseBracket),
                    &[CloseBracket],
                    |_| false),
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            Some((vec![], 0)),
            Some((vec![
                Some(Pattern::Abc(Spanned {
                    value: "abc",
                    span: Span::enclosing(Pos::new(5, 0, 5), Pos::new(8, 0, 8)),
                })),
                Some(Pattern::Abc(Spanned {
                    value: "abc",
                    span: Span::enclosing(Pos::new(10, 0, 10), Pos::new(13, 0, 13)),
                })),
            ], 0)),
            Some((vec![
                Some(Pattern::Xyc(Spanned {
                    value: "aac",
                    span: Span::enclosing(Pos::new(17, 0, 17), Pos::new(20, 0, 20)),
                })),
            ], 0)),
        ], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(22, 0, 22));
}


/// Test successful `list_bounded` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_list::bracket_list_commas -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn bracket_list_commas() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bracket_list_commas")
        .entered();
    let (lexer, ctx, errors, _source) = build_test_lexer("[,,,,,abc]");
    use AbcToken::*;

    let (value, succ) = bracket_default_index(
            &[OpenBracket],
            list_bounded(
                1, None,
                pattern,
                Comma,
                |tok| *tok == CloseBracket),
            &[CloseBracket],
            |_| false)
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = (vec![
            None,
            None,
            None,
            None,
            None,
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
            })),
        ], 0);

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));

    assert_eq!(errors.read().unwrap().len(), 5);
    assert_eq!(format!("{}", errors.read().unwrap().first().unwrap()), "\
error: expected pattern
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [,,,,,abc]
  |  \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
    assert_eq!(format!("{}", errors.read().unwrap().last().unwrap()), "\
error: expected pattern
 --> (0:0-0:10, bytes 0-10)
  | 
0 | [,,,,,abc]
  |      \\ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
