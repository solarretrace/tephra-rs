////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! ColumnMetrics tests.
////////////////////////////////////////////////////////////////////////////////
// NOTE: Run the following command to get tracing output:
// RUST_LOG=[test_name]=TRACE cargo test test_name -- --nocapture


// Local imports.
use tephra_span::ColumnMetrics;
use tephra_span::Pos;

// External library imports.
use pretty_assertions::assert_eq;
use test_log::test;

////////////////////////////////////////////////////////////////////////////////
// ColumnMetrics tests.
////////////////////////////////////////////////////////////////////////////////

/// Performs size checks.
#[allow(unused_qualifications)]
#[test]
#[tracing::instrument]
fn size_checks() {
    use std::mem::size_of;
    assert_eq!(2, size_of::<tephra_span::ColumnMetrics>(), "ColumnMetrics");
}

/// Tests `ColumnMetrics::position_after_str` for `Lf`.
#[test]
#[tracing::instrument]
fn lf_position_after_str() {
    let text = "abcd";
    let metrics = ColumnMetrics::new();


    let actual = metrics.position_after_str(text, Pos::ZERO, "ab");
    let expected = Some(Pos::new(2, 0, 2));
    assert_eq!(actual, expected);

    let actual = metrics.position_after_str(text, Pos::new(1, 2, 3), "bc");
    let expected = Some(Pos::new(3, 2, 5));
    assert_eq!(actual, expected);
}

/// Tests `ColumnMetrics::position_after_char_matching` for `Lf`.
#[test]
#[tracing::instrument]
fn lf_next_position_after_chars_matching() {
    let text = "    \t\tabcd";
    let metrics = ColumnMetrics::new();


    let actual = metrics.next_position_after_chars_matching(
        text,
        Pos::ZERO,
        char::is_whitespace);
    let expected = Some(Pos::new(1, 0, 1));
    assert_eq!(actual, expected);

    let actual = metrics.next_position_after_chars_matching(
        text,
        Pos::new(4, 2, 3),
        char::is_whitespace);
    let expected = Some(Pos::new(5, 2, 4));
    assert_eq!(actual, expected);
}

/// Tests `ColumnMetrics::position_after_chars_matching` for `Lf`.
#[test]
#[tracing::instrument]
fn lf_position_after_chars_matching() {
    let text = "    \t\tabcd";
    let metrics = ColumnMetrics::new();


    let actual = metrics.position_after_chars_matching(
        text,
        Pos::ZERO,
        char::is_whitespace);
    let expected = Some(Pos::new(6, 0, 12));
    assert_eq!(actual, expected);

    let actual = metrics.position_after_chars_matching(
        text,
        Pos::new(4, 2, 3),
        char::is_whitespace);
    let expected = Some(Pos::new(6, 2, 8));
    assert_eq!(actual, expected);
}

/// Tests `ColumnMetrics::iter_columns` for `Lf`.
#[test]
#[tracing::instrument]
fn lf_iter_columns() {
    let text = "abcd";
    let metrics = ColumnMetrics::new();


    let actual: Vec<_> = metrics.iter_columns(text, Pos::ZERO)
        .collect();

    let expected = vec![
        ("a", Pos::new(1, 0, 1)),
        ("b", Pos::new(2, 0, 2)),
        ("c", Pos::new(3, 0, 3)),
        ("d", Pos::new(4, 0, 4)),
    ];
    assert_eq!(actual, expected);
}
