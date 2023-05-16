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
use crate::test::abc::Pattern;
use crate::test::abc::Flex;
use crate::test::abc::flex;

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
//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_nested::flex_one -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn flex_one() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "flex_one")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[abc]");

    let (value, succ) = flex
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Flex::Block(Spanned {
        span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
        value: vec![
            Some(Flex::Pat(Pattern::Abc(Spanned {
                span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
                value: "abc",
            }))),
        ],
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(5, 0, 5));
}

//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_nested::flex_two -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn flex_two() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "flex_two")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[abc, bac]");

    let (value, succ) = flex
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Flex::Block(Spanned {
        span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(9, 0, 9)),
        value: vec![
            Some(Flex::Pat(Pattern::Abc(Spanned {
                span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(4, 0, 4)),
                value: "abc",
            }))),
            Some(Flex::Pat(Pattern::Xyc(Spanned {
                span: Span::enclosing(Pos::new(6, 0, 6), Pos::new(9, 0, 9)),
                value: "bac",
            }))),
        ],
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(10, 0, 10));
}


//
// To collect trace output:
// RUST_LOG=TRACE cargo test --all-features test::abc_nested::flex_nested -- --exact --nocapture > .trace
#[test]
#[timeout(100)]
fn flex_nested() {
    let _trace_guard = setup_test_environment();
    let _trace_span = span!(Level::DEBUG, "flex_nested")
        .entered();
    let (lexer, ctx, _errors, _source) = build_test_lexer("[[abc], [bac]]");

    let (value, succ) = flex
        (lexer.clone(), ctx)
        .expect("successful parse")
        .take_value();

    let actual = value;
    let expected = Flex::Block(Spanned {
        span: Span::enclosing(Pos::new(1, 0, 1), Pos::new(13, 0, 13)),
        value: vec![
            Some(Flex::Block(Spanned {
                span: Span::enclosing(Pos::new(2, 0, 2), Pos::new(5, 0, 5)),
                value: vec![
                    Some(Flex::Pat(Pattern::Abc(Spanned {
                        span: Span::enclosing(
                            Pos::new(2, 0, 2),
                            Pos::new(5, 0, 5)),
                        value: "abc",
                    }))),
                ],
            })),
            Some(Flex::Block(Spanned {
                span: Span::enclosing(Pos::new(9, 0, 9), Pos::new(12, 0, 12)),
                value: vec![
                    Some(Flex::Pat(Pattern::Xyc(Spanned {
                        span: Span::enclosing(
                            Pos::new(9, 0, 9),
                            Pos::new(12, 0, 12)),
                        value: "bac",
                    }))),
                ],
            })),
        ],
    });

    assert_eq!(actual, expected);
    assert_eq!(succ.lexer.cursor_pos(), Pos::new(14, 0, 14));
}
