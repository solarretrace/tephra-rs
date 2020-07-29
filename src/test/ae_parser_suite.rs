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
use crate::result::Success;
use crate::test::atma_expr::*;

////////////////////////////////////////////////////////////////////////////////
// Parser tests.
////////////////////////////////////////////////////////////////////////////////



/// Tests AtmaExprScanner with an empty single-quoted string.
#[test]
fn color() {
    let text = "#123456";
    let scanner = AtmaExprScanner::new();
    let mut lexer = Lexer::new(scanner, text, Lf);


    let actual =  parse_color(lexer).unwrap().value;
    let expected = Color(123456);

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
