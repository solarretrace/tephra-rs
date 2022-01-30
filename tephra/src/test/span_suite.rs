////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Span tests.
////////////////////////////////////////////////////////////////////////////////
// NOTE: Run the following command to get tracing output:
// RUST_LOG=[test_name]=TRACE cargo test test_name -- --nocapture


// Local imports.
use tephra_span::Span;
use tephra_span::ColumnMetrics;
use tephra_span::Pos;

// External library imports.
use pretty_assertions::assert_eq;
use test_log::test;

////////////////////////////////////////////////////////////////////////////////
// Span tests.
////////////////////////////////////////////////////////////////////////////////

/// Performs size checks.
#[allow(unused_qualifications)]
#[test]
#[tracing::instrument]
fn size_checks() {
    use std::mem::size_of;
    assert_eq!(64, size_of::<tephra_span::Span<'_>>(), "Span");
    assert_eq!(112, size_of::<tephra_span::SpanOwned>(), "SpanOwned");
}

/// Tests `Span::new`.
#[test]
#[tracing::instrument]
fn empty() {
    const TEXT: &'static str = "abcd";
    let span = Span::new(TEXT);

    assert_eq!(span.text(), "");
    assert_eq!(
        format!("{:?}", span),
        "\"\" (0:0, byte 0)");
}

/// Tests `Span::full`.
#[test]
#[tracing::instrument]
fn full() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let span = Span::full(TEXT, ColumnMetrics::new());

    assert_eq!(
        format!("{:?}", span),
        "\" \n  abcd  \n \" (0:0-2:1, bytes 0-12)");
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let span = Span::new_enclosing(
        TEXT,
        Pos::new(4, 1, 2),
        Pos::new(8, 1, 6));

    assert_eq!(
        format!("{:?}", span.widen_to_line(ColumnMetrics::new())),
        "\"  abcd  \" (1:0-1:8, bytes 2-10)");
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn widen_empty_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let span = Span::new_at(TEXT, Pos::new(6, 1, 4));

    assert_eq!(
        format!("{:?}", span.widen_to_line(ColumnMetrics::new())),
        "\"  abcd  \" (1:0-1:8, bytes 2-10)");
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn widen_line_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let span = Span::new_enclosing(
        TEXT,
        Pos::new(2, 1, 0),
        Pos::new(10, 1, 8));

    assert_eq!(
        format!("{:?}", span.widen_to_line(ColumnMetrics::new())),
        "\"  abcd  \" (1:0-1:8, bytes 2-10)");
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn widen_full_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let span = Span::new_enclosing(
        TEXT,
        Pos::new(0, 0, 0),
        Pos::new(12, 2, 1));

    assert_eq!(
        format!("{:?}", span.widen_to_line(ColumnMetrics::new())),
        "\" \n  abcd  \n \" (0:0-2:1, bytes 0-12)");
}


/// Tests `Span::split_lines`.
#[test]
#[tracing::instrument]
fn split_lines() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let span = Span::full(TEXT, ColumnMetrics::new());

    let actual = span
        .split_lines(ColumnMetrics::new())
        .map(|v| format!("{:?}", v))
        .collect::<Vec<_>>();
    let expected = vec![
        "\"\" (0:0, byte 0)".to_owned(),
        "\" \" (1:0-1:1, bytes 1-2)".to_owned(),
        "\"\" (2:0, byte 3)".to_owned(),
        "\" \" (3:0-3:1, bytes 4-5)".to_owned(),
        "\"abcd\" (4:0-4:4, bytes 6-10)".to_owned(),
        "\" def \" (5:0-5:5, bytes 11-16)".to_owned(),
        "\"ghi\" (6:0-6:3, bytes 17-20)".to_owned(),
        "\"\" (7:0, byte 21)".to_owned(),
    ];

    assert_eq!(actual, expected);
}


/// Tests `Span::split_lines` with no line breaks.
#[test]
#[tracing::instrument]
fn split_line_no_breaks() {
    const TEXT: &'static str = "abcd";
    let span = Span::full(TEXT, ColumnMetrics::new());

    let actual = span
        .split_lines(ColumnMetrics::new())
        .map(|v| format!("{:?}", v))
        .collect::<Vec<_>>();
    let expected = vec![
        "\"abcd\" (0:0-0:4, bytes 0-4)".to_owned(),
    ];

    assert_eq!(actual, expected);
}


/// Tests `Span::enclose`.
#[test]
#[tracing::instrument]
fn enclose() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let a = Span::new_enclosing(
        TEXT,
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        TEXT,
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));

    let actual = format!("{:?}", a.enclose(b));
    let expected = "\"\n \nabcd\n def \nghi\" (2:0-6:3, bytes 3-20)".to_owned();

    assert_eq!(actual, expected);
}

/// Tests `Span::union`.
#[test]
#[tracing::instrument]
fn union() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let a = Span::new_enclosing(
        TEXT,
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        TEXT,
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));

    let actual = format!("{:?}", a.union(b).next().unwrap());
    let expected = "\"\n \nabcd\n def \nghi\" (2:0-6:3, bytes 3-20)".to_owned();

    assert_eq!(actual, expected);
}

/// Tests `Span::intersect`.
#[test]
#[tracing::instrument]
fn intersect() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let a = Span::new_enclosing(
        TEXT,
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        TEXT,
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));

    let actual = format!("{:?}", a.intersect(b).unwrap());
    let expected = "\"\nabcd\" (3:1-4:4, bytes 5-10)".to_owned();

    assert_eq!(actual, expected);
}

/// Tests `Span::minus`.
#[test]
#[tracing::instrument]
fn minus() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let a = Span::new_enclosing(
        TEXT,
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        TEXT,
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));

    let actual = format!("{:?}", a.minus(b).next().unwrap());
    let expected = "\"\n \" (2:0-3:1, bytes 3-5)".to_owned();

    assert_eq!(actual, expected);
}
