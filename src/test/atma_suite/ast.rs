////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma AST tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::lexer::Lexer;
use crate::position::Lf;



////////////////////////////////////////////////////////////////////////////////
// PrimaryExpr
////////////////////////////////////////////////////////////////////////////////

/// Tests `primary_expr` with an Ident value.
#[test]
fn primary_expr_ident() {
    let text = "abcd";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Ident("abcd"),
        "\"abcd\" (0:0-0:4, bytes 0-4)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `primary_expr` with an Float value.
#[test]
fn primary_expr_float() {
    let text = "123.45";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Float("123.45"),
        "\"123.45\" (0:0-0:6, bytes 0-6)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an Uint value.
#[test]
fn primary_expr_uint() {
    let text = "12345";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Uint("12345"),
        "\"12345\" (0:0-0:5, bytes 0-5)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


/// Tests `primary_expr` with an Uint value with parens.
#[test]
fn primary_expr_uint_parens() {
    let text = "(12345)";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    // for tok in &mut lexer {
    //     println!("{:?}", tok);
    // }

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Tuple(vec![
            AstExpr::Unary(
                UnaryExpr::Call(
                    CallExpr::Primary(
                        PrimaryExpr::Uint("12345")))),
        ]),
        "\"(12345)\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
