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
use crate::span::Lf;
use crate::span::Pos;


////////////////////////////////////////////////////////////////////////////////
// Span tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests `Span::new`.
#[test]
fn span_empty() {
    let text = "abcd";
    let span: Span<'_, Lf> = Span::new(text);

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
    let mut span: Span<'_, Lf> = Span::new(text);
    span.extend_by_bytes(2);

    assert_eq!(
        format!("{}", span),
        "\"ab\" (0:0-0:2, bytes 0-2)");
}

/// Tests `Span::extend_by_bytes` with a newline.
#[test]
fn span_extend_by_bytes_newline() {
    let text = "a\nbcd";
    let mut span: Span<'_, Lf> = Span::new(text);
    span.extend_by_bytes(2);

    assert_eq!(
        format!("{}", span),
        "\"a\n\" (0:0-1:0, bytes 0-2)");
}

/// Tests `Span::extend_by_bytes` with a newline.
#[test]
fn span_extend_by_bytes_newline_split() {
    let text = "a\nbcd";
    let mut span: Span<'_, Lf> = Span::new(text);
    span.extend_by_bytes(3);

    assert_eq!(
        format!("{}", span),
        "\"a\nb\" (0:0-1:1, bytes 0-3)");
}


/// Tests `Span::new_from`.
#[test]
fn span_new_from() {
    let text = " \n  abcd  \n ";
    let mut span: Span<'_, Lf> = Span::new_from(Pos::new(4, 1, 2), text);
    span.extend_by(Pos::new(4, 0, 4));

    assert_eq!(
        format!("{}", span),
        "\"abcd\" (1:2-1:6, bytes 4-8)");
}


/// Tests `Span::widen_to_line`.
#[test]
fn span_widen_to_line() {
    let text = " \n  abcd  \n ";
    let mut span: Span<'_, Lf> = Span::new_from(Pos::new(4, 1, 2), text);
    span.extend_by(Pos::new(4, 0, 4));

    assert_eq!(
        format!("{}", span.widen_to_line()),
        "\"  abcd  \" (1:0-1:8, bytes 2-10)");
}

/// Tests `Span::widen_to_line`.
#[test]
fn span_widen_empty_to_line() {
    let text = " \n  abcd  \n ";
    let span: Span<'_, Lf> = Span::new_from(Pos::new(6, 1, 4), text);

    assert_eq!(
        format!("{}", span.widen_to_line()),
        "\"  abcd  \" (1:0-1:8, bytes 2-10)");
}


/// Tests `Span::widento_line`.
#[test]
fn span_widen_full_to_line() {
    let text = " \n  abcd  \n ";
    let mut span: Span<'_, Lf> = Span::new(text);
    span.extend_by(Pos::new_from_string::<_, Lf>(text));

    assert_eq!(
        format!("{}", span.widen_to_line()),
        "\" \n  abcd  \n \" (0:0-2:1, bytes 0-12)");
}


/// Tests `Span::trim`.
#[test]
fn span_trim() {
    let text = " \n  abcd  \n ";
    let mut span: Span<'_, Lf> = Span::new_from(Pos::new(2, 1, 0), text);
    span.extend_by(Pos::new(8, 0, 8));

    assert_eq!(
        format!("{}", span.trim()),
        "\"abcd\" (1:2-1:6, bytes 4-8)");
}

/// Tests `Span::split_lines`.
#[test]
fn span_split_lines() {
    let text = "\n \n\n \nabcd\n def \nghi\n";
    let span: Span<'_, Lf> = Span::new_full(text);

    let actual = span
        .split_lines()
        .map(|v| format!("{}", v))
        .collect::<Vec<_>>();
    let expected = vec![
        "\"\" (0:0-0:0, bytes 0-0)".to_owned(),
        "\" \" (1:0-1:1, bytes 1-2)".to_owned(),
        "\"\" (2:0-2:0, bytes 3-3)".to_owned(),
        "\" \" (3:0-3:1, bytes 4-5)".to_owned(),
        "\"abcd\" (4:0-4:4, bytes 6-10)".to_owned(),
        "\" def \" (5:0-5:5, bytes 11-16)".to_owned(),
        "\"ghi\" (6:0-6:3, bytes 17-20)".to_owned(),
        "\"\" (7:0-7:0, bytes 21-21)".to_owned(),
    ];

    for (i, act) in actual.iter().enumerate() {
        println!("{:?}", act);
        println!("{:?}", expected[i]);
        println!();
    }
    assert_eq!(actual, expected);
}
