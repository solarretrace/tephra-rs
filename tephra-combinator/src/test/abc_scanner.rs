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
// Misc tests
////////////////////////////////////////////////////////////////////////////////

/// Tests Abc token lexing & filtering.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_scanner::abc_tokens -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn abc_tokens() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "abc_tokens")
        .entered();
    let (mut lexer, _ctx, _errors, source) = build_test_lexer("a b\nc d");
    use AbcToken::*;

    let actual = lexer
        .iter_with_spans()
        .map(|lex| (
            lex.0,
            format!("{:?} ({})", source.clipped(lex.1).as_str(), lex.1)))
        .collect::<Vec<_>>();

    let expected = vec![
        (A,   "\"a\" (0:0-0:1, bytes 0-1)".to_string()),
        (B,   "\"b\" (0:2-0:3, bytes 2-3)".to_string()),
        (C,   "\"c\" (1:0-1:1, bytes 4-5)".to_string()),
        (D,   "\"d\" (1:2-1:3, bytes 6-7)".to_string()),
    ];

    assert_eq!(actual, expected);
}

/// Parses a `Pattern::Abc`.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_scanner::abc_pattern -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn abc_pattern() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "abc_pattern")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("abc");

    let (value, succ) = pattern
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Abc(Spanned {
        value: "abc",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Bxx`.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_scanner::bxx_pattern -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn bxx_pattern() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "bxx_pattern")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("baa");

    let (value, succ) = pattern
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Bxx(Spanned {
        value: "baa",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Parses a `Pattern::Xyc`.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_scanner::xyc_pattern -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn xyc_pattern() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "xyc_pattern")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("bac");

    let (value, succ) = pattern
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Pattern::Xyc(Spanned {
        value: "bac",
        span: source.full_span(),
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(3, 0, 3));
}

/// Ensures that a failure encountered after initial newline & whitespace
/// doesn't include that whitespace in the error message.
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_scanner::initial_newline_ws_skip -- --exact --nocapture > .trace
#[test]
#[timeout(50)]
fn initial_newline_ws_skip() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "initial_newline_ws_skip")
        .entered();
    let (lexer, ctx, _errors, source) = build_test_lexer("\n    aaa");

    let actual = pattern
        (lexer.clone(), ctx)
        .map_err(|e| e.into_source_error(source))
        .unwrap_err();

    assert_eq!(format!("{actual}"), "\
error: expected pattern
 --> (1:0-1:7, bytes 1-8)
  | 
1 |     aaa
  |     ^^^ expected 'ABC', 'BXX', or 'XYC' pattern
");
}
