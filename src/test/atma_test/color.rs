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



////////////////////////////////////////////////////////////////////////////////
// Hex code tests.
////////////////////////////////////////////////////////////////////////////////

/// Tests hex_code with 3 digits.
#[test]
fn hex_code_3() {
    let text = "#abc";
    let scanner = AtmaScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let actual = hex_code(3)
        (lexer)
        .unwrap()
        .value_span_display();

    // let expected = (0xABC,  "\"#abc\" (0:0-0:4, bytes 0-4)".to_owned());

    println!("{:?}", actual);
    // println!("{:?}", expected);
    // println!("");

    // assert_eq!(actual, expected);
}
