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
use crate::result::Reason;
use crate::test::atma_expr::*;

////////////////////////////////////////////////////////////////////////////////
// Parser tests.
////////////////////////////////////////////////////////////////////////////////


/// Tests AtmaExprScanner with an empty single-quoted string.
#[test]
fn color() {
    let text = "#123456";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);


    let actual =  parse_color(lexer).unwrap().value_span_display();
    let expected = (Color(123456), "\"#123456\" (0:0-0:7, bytes 0-7)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}

/// Tests AtmaExprScanner with an empty single-quoted string.
#[test]
fn color_too_long() {
    let text = "#1234567";
    let scanner = AtmaExprScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let failure = parse_color(lexer).err().unwrap();
    println!("{}", failure);

    let actual = failure.error_span_display();
    let expected = (Reason::IncompleteParse { 
                        context: "Color requires 6 hex digits".into(),
                    },
                    "\"#1234567\" (0:0-0:8, bytes 0-8)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
