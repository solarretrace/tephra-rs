////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Span tests.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::Pos;
use crate::SourceText;
use crate::Span;

// External library imports.
use pretty_assertions::assert_eq;


////////////////////////////////////////////////////////////////////////////////
// Basic Span tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Span::new`.
#[test]
#[should_panic]
fn span_offset_oob_empty() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = Span::new();

    // Clipping outside of the text should panic.
    source.clipped(span);
}

/// Tests `Span::new`.
#[test]
fn span_offset_within_empty() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = Span::new_at(Pos::new(100, 10, 10));

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "10:10, byte 100";
    assert_eq!(actual, expected);
}

/// Tests `SourceText::full_span`.
#[test]
fn source_text_offset_full_span() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = source.full_span();
    println!("{:?}", span);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = " \n  abcd  \n ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "10:10-12:1, bytes 100-112";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
fn span_offset_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = Span::new_enclosing(
            Pos::new(104, 11, 2),
            Pos::new(108, 11, 6))
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "  abcd  ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "11:0-11:8, bytes 102-110";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
fn span_offset_empty_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = Span::new_at(Pos::new(106, 11, 4))
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "  abcd  ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "11:0-11:8, bytes 102-110";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
fn span_offset_line_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = Span::new_enclosing(
            Pos::new(102, 11, 0),
            Pos::new(110, 11, 8))
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "  abcd  ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "11:0-11:8, bytes 102-110";
    assert_eq!(actual, expected);
}

/// Tests `Span::widen_to_line`.
#[test]
fn span_offset_full_widen_to_line() {
    const TEXT: &'static str = " \n  abcd  \n ";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = source.full_span()
        .widen_to_line(source);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = " \n  abcd  \n ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "10:10-12:1, bytes 100-112";
    assert_eq!(actual, expected);
}


/// Tests `Span::split_lines`.
#[test]
fn span_offset_split_lines() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = source.full_span();

    let actual = span
        .split_lines(source)
        .map(|sp| format!("{:?} ({})", source.clipped(sp).as_ref(), sp))
        .collect::<Vec<_>>();
    let expected = vec![
        "\"\" (10:10, byte 100)".to_owned(),
        "\" \" (11:0-11:1, bytes 101-102)".to_owned(),
        "\"\" (12:0, byte 103)".to_owned(),
        "\" \" (13:0-13:1, bytes 104-105)".to_owned(),
        "\"abcd\" (14:0-14:4, bytes 106-110)".to_owned(),
        "\" def \" (15:0-15:5, bytes 111-116)".to_owned(),
        "\"ghi\" (16:0-16:3, bytes 117-120)".to_owned(),
        "\"\" (17:0, byte 121)".to_owned(),
    ];

    assert_eq!(actual, expected);
}


/// Tests `Span::split_lines` with no line breaks.
#[test]
fn span_offset_no_breaks_split_line() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));
    let span = source.full_span();

    let actual = span
        .split_lines(source)
        .map(|sp| format!("{:?} ({})", source.clipped(sp).as_ref(), sp))
        .collect::<Vec<_>>();
    let expected = vec![
        "\"abcd\" (10:10-10:14, bytes 100-104)".to_owned(),
    ];

    assert_eq!(actual, expected);
}


/// Tests `Span::enclose`.
#[test]
fn span_offset_enclose() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));

    let a = Span::new_enclosing(
        Pos::new(103, 12, 0),
        Pos::new(110, 14, 4));
    let b = Span::new_enclosing(
        Pos::new(105, 13, 1),
        Pos::new(120, 16, 3));
    let span = a.enclose(b);

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\n \nabcd\n def \nghi";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "12:0-16:3, bytes 103-120";
    assert_eq!(actual, expected);
}

/// Tests `Span::union`.
#[test]
fn span_offset_union() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));

    let a = Span::new_enclosing(
        Pos::new(103, 12, 0),
        Pos::new(110, 14, 4));
    let b = Span::new_enclosing(
        Pos::new(105, 13, 1),
        Pos::new(120, 16, 3));
    let span = a.union(b).next().unwrap();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\n \nabcd\n def \nghi";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "12:0-16:3, bytes 103-120";
    assert_eq!(actual, expected);
}

/// Tests `Span::intersect`.
#[test]
fn span_offset_intersect() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));

    let a = Span::new_enclosing(
        Pos::new(103, 12, 0),
        Pos::new(110, 14, 4));
    let b = Span::new_enclosing(
        Pos::new(105, 13, 1),
        Pos::new(120, 16, 3));
    let span = a.intersect(b).unwrap();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\nabcd";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "13:1-14:4, bytes 105-110";
    assert_eq!(actual, expected);
}

/// Tests `Span::minus`.
#[test]
fn span_offset_minus() {
    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));

    let a = Span::new_enclosing(
        Pos::new(103, 12, 0),
        Pos::new(110, 14, 4));
    let b = Span::new_enclosing(
        Pos::new(105, 13, 1),
        Pos::new(120, 16, 3));
    let span = a.minus(b).next().unwrap();

    // Check text clip.
    let actual = source.clipped(span);
    let expected = "\n ";
    assert_eq!(actual.as_ref(), expected);

    // Check span display.
    let actual = format!("{}", span);
    let expected = "12:0-13:1, bytes 103-105";
    assert_eq!(actual, expected);
}



/// Tests `SourceText::iter_columns` for `Lf`.
#[test]
fn source_text_lf_iter_columns() {
    const TEXT: &'static str = "abcd";
    let source = SourceText::new(TEXT)
        .with_start_position(Pos::new(100, 10, 10));

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
