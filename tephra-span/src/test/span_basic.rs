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
// RUST_LOG=TRACE cargo test --features trace -- --show-output > .trace


// Internal library imports.
use crate::Pos;
use crate::SourceText;
use crate::Span;

// External library imports.
use pretty_assertions::assert_eq;
use test_log::test;



////////////////////////////////////////////////////////////////////////////////
// Basic Span tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Span::new`.
#[test]
#[tracing::instrument]
fn span_basic_empty() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT);
    let span = Span::new();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "0:0, byte 0";
    assert_eq!(actual, expected);
}

/// Tests `SourceText::full_span`.
#[test]
#[tracing::instrument]
fn source_text_basic_full_span() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT);
    let span = source.full_span();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = " \n  abcd  \n ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "0:0-2:1, bytes 0-12";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn span_basic_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT);
    let span = Span::new_enclosing(
            Pos::new(4, 1, 2),
            Pos::new(8, 1, 6))
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "  abcd  ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "1:0-1:8, bytes 2-10";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn span_basic_empty_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT);
    let span = Span::new_at(Pos::new(6, 1, 4))
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "  abcd  ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "1:0-1:8, bytes 2-10";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn span_basic_line_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT);
    let span = Span::new_enclosing(
            Pos::new(2, 1, 0),
            Pos::new(10, 1, 8))
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "  abcd  ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "1:0-1:8, bytes 2-10";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
#[tracing::instrument]
fn span_basic_full_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT);
    let span = source.full_span()
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = " \n  abcd  \n ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "0:0-2:1, bytes 0-12";
    assert_eq!(actual, expected);
}


/// Tests `Span::split_lines`.
#[test]
#[tracing::instrument]
fn span_basic_split_lines() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);
    let span = source.full_span();

    let actual = span
        .split_lines(source)
        .map(|sp| format!("{:?} ({})", source.clipped(sp).as_ref(), sp))
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
fn span_basic_no_breaks_split_line() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT);
    let span = source.full_span();

    let actual = span
        .split_lines(source)
        .map(|sp| format!("{:?} ({})", source.clipped(sp).as_ref(), sp))
        .collect::<Vec<_>>();
    let expected = vec![
        "\"abcd\" (0:0-0:4, bytes 0-4)".to_owned(),
    ];

    assert_eq!(actual, expected);
}


/// Tests `Span::enclose`.
#[test]
#[tracing::instrument]
fn span_basic_enclose() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);

    let a = Span::new_enclosing(
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));
    let span = a.enclose(b);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\n \nabcd\n def \nghi";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "2:0-6:3, bytes 3-20";
    assert_eq!(actual, expected);
}

/// Tests `Span::union`.
#[test]
#[tracing::instrument]
fn span_basic_union() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);

    let a = Span::new_enclosing(
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));
    let span = a.union(b).next().unwrap();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\n \nabcd\n def \nghi";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "2:0-6:3, bytes 3-20";
    assert_eq!(actual, expected);
}

/// Tests `Span::intersect`.
#[test]
#[tracing::instrument]
fn span_basic_intersect() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);

    let a = Span::new_enclosing(
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));
    let span = a.intersect(b).unwrap();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\nabcd";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "3:1-4:4, bytes 5-10";
    assert_eq!(actual, expected);
}

/// Tests `Span::minus`.
#[test]
#[tracing::instrument]
fn span_basic_minus() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);

    let a = Span::new_enclosing(
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));
    let b = Span::new_enclosing(
        Pos::new(5, 3, 1),
        Pos::new(20, 6, 3));
    let span = a.minus(b).next().unwrap();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\n ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "2:0-3:1, bytes 3-5";
    assert_eq!(actual, expected);
}



/// Tests `SourceText::iter_columns` for `Lf`.
#[test]
#[tracing::instrument]
fn source_text_lf_iter_columns() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT);

    let actual: Vec<_> = source.iter_columns(Pos::ZERO)
        .collect();

    let expected = vec![
        ("a", Pos::new(1, 0, 1)),
        ("b", Pos::new(2, 0, 2)),
        ("c", Pos::new(3, 0, 3)),
        ("d", Pos::new(4, 0, 4)),
    ];
    assert_eq!(actual, expected);
}
