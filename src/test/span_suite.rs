////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Span tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::span::Span;


////////////////////////////////////////////////////////////////////////////////
// Span tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Span::new`.
#[test]
fn span_empty() {
    let text = "abcd";
    let span = Span::new(text);

    assert_eq!(
        span.text(),
        "");
    assert_eq!(
        format!("{}", span),
        "\"\" (0:0-0:0, bytes 0-0)");
}

/// Tests `Span::extend_by_bytes`.
#[test]
fn span_extend_by_bytes() {
    let text = "abcd";
    let mut span = Span::new(text);
    span.extend_by_bytes(2, "\n");

    assert_eq!(
        format!("{}", span),
        "\"ab\" (0:0-0:2, bytes 0-2)");
}

/// Tests `Span::extend_by_bytes` with a newline.
#[test]
fn span_extend_by_bytes_newline() {
    let text = "a\nbcd";
    let mut span = Span::new(text);
    span.extend_by_bytes(2, "\n");

    assert_eq!(
        format!("{}", span),
        "\"a\n\" (0:0-1:0, bytes 0-2)");
}

/// Tests `Span::extend_by_bytes` with a newline.
#[test]
fn span_extend_by_bytes_newline_split() {
    let text = "a\nbcd";
    let mut span = Span::new(text);
    span.extend_by_bytes(3, "\n");

    assert_eq!(
        format!("{}", span),
        "\"a\nb\" (0:0-1:1, bytes 0-3)");
}
