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
    let expected = (Color(123456), "\"#123456\" (0:0-0:7, bytes 0-7)".to_owned());

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

