////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma color tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::lexer::Lexer;
use crate::span::Lf;

// External library imports.
use ::color::Rgb;
use ::color::Color;


////////////////////////////////////////////////////////////////////////////////
// RGB hex code
////////////////////////////////////////////////////////////////////////////////

/// Tests `rgb_hex_code` with 3 digits.
#[test]
fn rgb_hex_code_3() {
    let text = "#abc";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual = rgb_hex_code
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        Rgb::from(0xAABBCC),
        "\"#abc\" (0:0-0:4, bytes 0-4)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `rgb_hex_code` with 2 digits, expecting 3.
#[test]
fn rgb_hex_code_3_short() {
    let text = "#ab";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let failure = rgb_hex_code
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid color code",
        "\"#ab\" (0:0-0:3, bytes 0-3)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `rgb_hex_code` with 4 digits, expecting 3.
#[test]
fn rgb_hex_code_3_long() {
    let text = "#abcd";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let failure = rgb_hex_code
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid color code",
        "\"#abcd\" (0:0-0:5, bytes 0-5)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `rgb_hex_code` with 3 digits, with unexpected whitespace.
#[test]
fn rgb_hex_code_3_whitespace() {
    let text = "# abc";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok == AtmaToken::Whitespace);

    let failure = rgb_hex_code
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "unexpected token",
        "\"# \" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `rgb_hex_code` with 3 digits.
#[test]
fn rgb_hex_code_6() {
    let text = "#abcdef";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual = rgb_hex_code
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        Rgb::from(0xABCDEF), 
        "\"#abcdef\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `rgb_hex_code` with 2 digits, expecting 3.
#[test]
fn rgb_hex_code_6_short() {
    let text = "#abcde";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let failure = rgb_hex_code
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid color code",
        "\"#abcde\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `rgb_hex_code` with 4 digits, expecting 3.
#[test]
fn rgb_hex_code_6_long() {
    let text = "#abcdefa";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let failure = rgb_hex_code
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid color code",
        "\"#abcdefa\" (0:0-0:8, bytes 0-8)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `rgb_hex_code` with 3 digits, with unexpected whitespace.
#[test]
fn rgb_hex_code_6_whitespace() {
    let text = "# abcdef";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let failure = rgb_hex_code
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "unexpected token",
        "\"# \" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// RGB function
////////////////////////////////////////////////////////////////////////////////


/// Tests `color_function` with RGB u8 values.
#[test]
fn function_rgb_u8() {
    let text = "rgb(0,255,127)";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = color_function
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        Color::from(Rgb::from([0, 255, 127])),
        "\"rgb(0,255,127)\" (0:0-0:14, bytes 0-14)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `color_function` with RGB u8 values and newlines.
#[test]
fn function_rgb_u8_newline() {
    let text = "rgb(0, \n\
        255, \n\
        127)";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = color_function
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        Color::from(Rgb::from([0, 255, 127])),
        "\"rgb(0, \n255, \n127)\" (0:0-2:4, bytes 0-18)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `color_function` with RGB u8 values, using the wrong bracket.
#[test]
fn function_rgb_u8_wrong_bracket() {
    let text = "rgb[0,255,127]";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let failure = color_function
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "unexpected token",
        "\"[\" (0:3-0:4, bytes 3-4)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `color_function` with RGB u8 values, out of range.
#[test]
fn function_rgb_u8_out_of_range() {
    let text = "rgb(0, 255, 1270)";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = color_function
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid RGB color",
        "\"rgb(0, 255, 1270)\" (0:0-0:17, bytes 0-17)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `color_function` with RGB u8 values with newlines, out of range.
#[test]
fn function_rgb_u8_out_of_range_newline() {
    let text = "rgb(0,\n\
        255,\n\
        1270)";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let failure = color_function
        (lexer)
        .err()
        .unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();

    let expected = (
        "invalid RGB color",
        "\"rgb(0,\n255,\n1270)\" (0:0-2:5, bytes 0-17)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
