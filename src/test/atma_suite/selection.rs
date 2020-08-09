////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma selection and cell ref tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::lexer::Lexer;
use crate::span::Lf;


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

/// Tests `cell_ref` with an index value.
#[test]
fn cell_ref_index() {
    let text = ":0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_ref
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellRef::Index(0),
        "\":0\" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_ref` with an index value out of range.
#[test]
fn cell_ref_index_out_of_range() {
    let text = ":0x100000000";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_ref
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\":0x100000000\" (0:0-0:12, bytes 0-12)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_ref` with an position value.
#[test]
fn cell_ref_position() {
    let text = ":0.0.0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_ref
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellRef::Position(Position { page: 0, line: 0, column: 0 }),
        "\":0.0.0\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_ref` with an position value out of range.
#[test]
fn cell_ref_position_out_of_range() {
    let text = ":0xFFFF.0x10000.0xFFFF";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_ref
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\":0xFFFF.0x10000\" (0:0-0:15, bytes 0-15)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_ref` with an name value.
#[test]
fn cell_ref_name() {
    let text = "'abcd'";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_ref
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellRef::Name("abcd".into()),
        "\"'abcd'\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_ref` with an group value.
#[test]
fn cell_ref_group() {
    let text = "'abcd':0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_ref
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellRef::Group { group: "abcd".into(), idx: 0 },
        "\"'abcd':0\" (0:0-0:8, bytes 0-8)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_ref` with an group value out of range.
#[test]
fn cell_ref_group_out_of_range() {
    let text = "'abcd':0x1_0000_0000";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_ref
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\"'abcd':0x1_0000_0000\" (0:0-0:20, bytes 0-20)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// CellSelector::All
////////////////////////////////////////////////////////////////////////////////

/// Tests `cell_selector` with an All value.
#[test]
fn cell_selector_all() {
    let text = ":*";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_selector
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellSelector::All,
        "\":*\" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// CellSelector::Index
////////////////////////////////////////////////////////////////////////////////
/// Tests `cell_selector` with an Index value.
#[test]
fn cell_selector_index() {
    let text = ":0xFFFFFFFF";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_selector
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellSelector::Index(0xFFFFFFFF),
        "\":0xFFFFFFFF\" (0:0-0:11, bytes 0-11)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with an Index value out of range.
#[test]
fn cell_selector_index_out_of_range() {
    let text = ":0x1_0000_0000";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_selector
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\":0x1_0000_0000\" (0:0-0:14, bytes 0-14)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// CellSelector::IndexRange
////////////////////////////////////////////////////////////////////////////////

/// Tests `cell_selector` with an IndexRange value.
#[test]
fn cell_selector_index_range() {
    let text = ":0xA00F - :0xFFFF";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let actual = cell_selector
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellSelector::IndexRange { low: 0xA00F, high: 0xFFFF },
        "\":0xA00F - :0xFFFF\" (0:0-0:17, bytes 0-17)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with an IndexRange value out of range.
#[test]
fn cell_selector_index_range_out_of_range() {
    let text = ":0-:0x1_0000_0000";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_selector
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\":0-:0x1_0000_0000\" (0:0-0:17, bytes 0-17)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with an IndexRange value out of order.
#[test]
fn cell_selector_index_range_out_of_order() {
    let text = ":0xFF - :0b10";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_selector
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid index range",
        "\":0xFF - :0b10\" (0:0-0:13, bytes 0-13)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// CellSelector::PositionSelector
////////////////////////////////////////////////////////////////////////////////

/// Tests `cell_selector` with a PositionSelector value.
#[test]
fn cell_selector_position_selector() {
    let text = ":0xFFFF.*.0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_selector
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellSelector::PositionSelector(PositionSelector {
            page: Some(0xFFFF),
            line: None,
            column: Some(0),
        }),
        "\":0xFFFF.*.0\" (0:0-0:11, bytes 0-11)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with a PositionSelector value out of range.
#[test]
fn cell_selector_position_selector_out_of_range() {
    let text = ":0x1_0000.*.0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_selector
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\":0x1_0000\" (0:0-0:9, bytes 0-9)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with a PositionRange value.
#[test]
fn cell_selector_position_range() {
    let text = ":0xFFFF.0.0 - :0xFFFF.2.3";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = cell_selector
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CellSelector::PositionRange { 
            low: Position {
                page: 0xFFFF, line: 0, column: 0,
            },
            high: Position {
                page: 0xFFFF, line: 2, column: 3,
            },
        },
        "\":0xFFFF.0.0 - :0xFFFF.2.3\" (0:0-0:25, bytes 0-25)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with a PositionRange value out of range.
#[test]
fn cell_selector_position_range_out_of_range() {
    let text = ":0xFFFF.0.0 - :0xFFFF.0x1_0000.3";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_selector
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid integer value",
        "\":0xFFFF.0.0 - :0xFFFF.0x1_0000\" (0:0-0:30, bytes 0-30)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `cell_selector` with a PositionRange value out of order.
#[test]
fn cell_selector_position_range_out_of_order() {
    let text = ":0xFFFF.1.0 -\n:0xFFFF.0.3";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = cell_selector
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid position range",
        "\":0xFFFF.1.0 -\n:0xFFFF.0.3\" (0:0-1:11, bytes 0-25)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
