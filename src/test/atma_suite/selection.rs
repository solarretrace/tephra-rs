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
        "index out of allowed range",
        "\":0x100000000\" (0:0-0:12, bytes 0-12)".to_owned());

    println!("{:?}", actual);
    println!("{:?}", expected);
    println!("");

    assert_eq!(actual, expected);
}
