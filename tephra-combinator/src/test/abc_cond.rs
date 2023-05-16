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
use crate::list;
use crate::implies;
use crate::sub;
use crate::test::abc::Abc;
use crate::test::abc::AbcToken;
use crate::test::abc::pattern;
use crate::test::abc::Pattern;

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
    let ctx = Context::empty();

    (lexer, ctx, errors, source)
}


////////////////////////////////////////////////////////////////////////////////
// Combinator tests
////////////////////////////////////////////////////////////////////////////////

/// Test successful `implies` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_cond::pattern_implies -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_implies() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_implies")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("abc bdd");

    let (value, succ) = implies(pattern, pattern)
        (lexer, ctx)
        .expect("successful parse")
        .take_value();


    let actual = value;
    let expected = Some((
        Pattern::Abc(Spanned {
            value: "abc",
            span: Span::enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
        }),
        Pattern::Bxx(Spanned {
            value: "bdd",
            span: Span::enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
        }),
    ));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(7, 0, 7));
}

/// Test successful `implies` combinator.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_cond::pattern_implies_failed_left -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_implies_failed_left() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_implies_failed_left")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("agc cdd");

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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_cond::pattern_implies_failed_right -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_implies_failed_right() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_implies_failed_right")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("abc cdd");

    let actual = implies(pattern, sub(pattern))
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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_cond::pattern_implies_list -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn pattern_implies_list() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "pattern_implies_list")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("abc[bdd, abc]");
    use AbcToken::*;

    let (value, succ) = implies(
            pattern,
            bracket(
                &[OpenBracket],
                list(
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
            span: Span::enclosing(Pos::new(0, 0, 0), Pos::new(3, 0, 3)),
        }),
        Some(vec![
            Some(Pattern::Bxx(Spanned {
                value: "bdd",
                span: Span::enclosing(Pos::new(4, 0, 4), Pos::new(7, 0, 7)),
            })),
            Some(Pattern::Abc(Spanned {
                value: "abc",
                span: Span::enclosing(Pos::new(9, 0, 9), Pos::new(12, 0, 12)),
            })),
        ]),
    ));

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(13, 0, 13));
}
