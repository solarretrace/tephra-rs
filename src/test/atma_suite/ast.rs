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

// External crate imports.
use ::color::Color;
use ::color::Rgb;


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


/// Tests `primary_expr` with an Uint value with brackets.
#[test]
fn primary_expr_uint_brackets() {
    let text = "[12345]";
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
        PrimaryExpr::Array(vec![
            AstExpr::Unary(
                UnaryExpr::Call(
                    CallExpr::Primary(
                        PrimaryExpr::Uint("12345")))),
        ]),
        "\"[12345]\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an Color value.
#[test]
fn primary_expr_color() {
    let text = "#ABCDEF";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::Color(Color::from(Rgb::from(0xABCDEF))),
        "\"#ABCDEF\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests `primary_expr` with an CellRef value.
#[test]
fn primary_expr_cell_ref() {
    let text = ":0";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = primary_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        PrimaryExpr::CellRef(CellRef::Index(0)),
        "\":0\" (0:0-0:2, bytes 0-2)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}


////////////////////////////////////////////////////////////////////////////////
// CallExpr
////////////////////////////////////////////////////////////////////////////////

/// Tests `call_expr` with an Call value.
#[test]
fn call_expr_call() {
    let text = "abcd( a, #12AB99, 12.34 )";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = call_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        CallExpr::Call(
            PrimaryExpr::Ident("abcd"),
            vec![
                AstExpr::Unary(
                    UnaryExpr::Call(
                        CallExpr::Primary(
                            PrimaryExpr::Ident("a")))),
                AstExpr::Unary(
                    UnaryExpr::Call(
                        CallExpr::Primary(
                            PrimaryExpr::Color(
                                Color::from(Rgb::from(0x12AB99)))))),
                AstExpr::Unary(
                    UnaryExpr::Call(
                        CallExpr::Primary(
                            PrimaryExpr::Float("12.34")))),
            ]),
        "\"abcd( a, #12AB99, 12.34 )\" (0:0-0:25, bytes 0-25)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}



////////////////////////////////////////////////////////////////////////////////
// AstExpr
////////////////////////////////////////////////////////////////////////////////

/// Tests `call_expr` with an Call value.
#[test]
fn ast_expr_call_negate() {
    let text = "-(abcd)( \na, \n#12AB99, \n12.34 )";
    let scanner = AtmaScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf::with_tab_width(4));
    lexer.set_filter_fn(|tok| *tok != AtmaToken::Whitespace);

    let actual = ast_expr
        (lexer)
        .unwrap()
        .value_span_display();

    let expected = (
        AstExpr::Unary(
            UnaryExpr::Minus(Box::new(UnaryExpr::Call(
                CallExpr::Call(
                    PrimaryExpr::Tuple(vec![
                        AstExpr::Unary(
                            UnaryExpr::Call(
                                CallExpr::Primary(
                                    PrimaryExpr::Ident("abcd"))))
                    ]),
                    vec![
                        AstExpr::Unary(
                            UnaryExpr::Call(
                                CallExpr::Primary(
                                    PrimaryExpr::Ident("a")))),
                        AstExpr::Unary(
                            UnaryExpr::Call(
                                CallExpr::Primary(
                                    PrimaryExpr::Color(
                                        Color::from(Rgb::from(0x12AB99)))))),
                        AstExpr::Unary(
                            UnaryExpr::Call(
                                CallExpr::Primary(
                                    PrimaryExpr::Float("12.34")))),
                    ]))))),
        "\"-(abcd)( \na, \n#12AB99, \n12.34 )\" (0:0-3:7, bytes 0-31)"
            .to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
