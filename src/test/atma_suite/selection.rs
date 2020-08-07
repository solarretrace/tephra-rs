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
    lexer.set_filter_fn(|tok| *tok == AtmaToken::Whitespace);


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
    lexer.set_filter_fn(|tok| *tok == AtmaToken::Whitespace);

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
    lexer.set_filter_fn(|tok| *tok == AtmaToken::Whitespace);

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
    lexer.set_filter_fn(|tok| *tok == AtmaToken::Whitespace);

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

