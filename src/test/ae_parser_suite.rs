////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! AtmaExpr parser tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::span::Lf;
use crate::lexer::Lexer;
use crate::test::atma_expr::*;

////////////////////////////////////////////////////////////////////////////////
// Parser tests.
////////////////////////////////////////////////////////////////////////////////


/// Tests `Color` parsing.
#[test]
fn color() {
    let text = "#123456";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_color(lexer).unwrap().value_span_display();
    let expected = (
        Color(123456),
        "\"#123456\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}

/// Tests `Color` parsing with extra digits.
#[test]
fn color_too_long() {
    let text = "#1234567";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let failure = parse_color(lexer).err().unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();
    let expected = ("invalid color",
                    "\"#1234567\" (0:0-0:8, bytes 0-8)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}

/// Tests `Color` parsing with extra whitespace inside.
#[test]
fn color_whitespace() {
    let text = "#  123456";
    let scanner = AtmaExprScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let failure = parse_color(lexer).err().unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();
    let expected = ("unexpected token",
                    "\"#  \" (0:0-0:3, bytes 0-3)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}



/// Tests `Channel` parsing.
#[test]
fn channel() {
    let text = "rgb";
    let scanner = AtmaExprScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual =  parse_channel(lexer).unwrap().value_span_display();
    let expected = (Channel::Rgb, "\"rgb\" (0:0-0:3, bytes 0-3)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}


/// Tests `Channel` parsing with an invalid value.
#[test]
fn channel_invalid() {
    let text = "bad";
    let scanner = AtmaExprScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let failure = parse_channel(lexer).err().unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();
    let expected = ("invalid channel",
                    "\"bad\" (0:0-0:3, bytes 0-3)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}



/// Tests `uint` parsing with u16 values.
#[test]
fn uint_16() {
    let text = "100";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_uint::<_, u16>(lexer).unwrap().value_span_display();
    let expected = (100u16, "\"100\" (0:0-0:3, bytes 0-3)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}


/// Tests `uint` parsing with u16 values.
#[test]
fn uint_16_hex() {
    let text = "0x1_F0";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_uint::<_, u16>(lexer).unwrap().value_span_display();
    let expected = (0x1_F0u16, "\"0x1_F0\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}


/// Tests single-quoted string parsing.
#[test]
fn string_single() {
    let text = "'abc\tdef'";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_string(lexer).unwrap().value_span_display();
    let expected = (
        "abc\tdef".into(),
        "\"'abc\tdef'\" (0:0-0:9, bytes 0-9)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}

/// Tests single-quoted string parsing with escaped characters.
#[test]
fn string_single_escaped() {
    let text = "'abc\\tdef'";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_string(lexer).unwrap().value_span_display();
    let expected = (
        "abc\tdef".into(),
        "\"'abc\\tdef'\" (0:0-0:10, bytes 0-10)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}

/// Tests `index` parsing.
#[test]
fn index() {
    let text = ":1";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_index(lexer).unwrap().value_span_display();
    let expected = (
        CellRef::Index(1),
        "\":1\" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}


/// Tests `position` parsing.
#[test]
fn position() {
    let text = ":1.3.5";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual =  parse_position(lexer).unwrap().value_span_display();
    let expected = (
        CellRef::Position(1, 3, 5),
        "\":1.3.5\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!();

    assert_eq!(actual, expected);
}
