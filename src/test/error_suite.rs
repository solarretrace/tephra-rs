////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error handling tests.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::lexer::Lexer;
use crate::primitive::one;
use crate::result::Failure;
use crate::result::ParseError;
use crate::span::Lf;
use crate::span::Span;
use crate::test::atma_script::*;


/// Tests `Failure` state for an unexpected empty parse.
#[test]
fn parse_one_from_empty() {
    let text = "";
    let scanner = AtmaScriptScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let res = one(AtmaToken::CommandChunk)(lexer.clone());

    assert_eq!(
        res,
        Err(Failure {
            lexer: lexer,
            parse_error: ParseError::unexpected_end_of_text(Span::new(text)),
            source: None,
        }));
}

/// Tests `Failure` message for an unexpected empty parse.
#[test]
fn empty_msg() {
    let text = "";
    let scanner = AtmaScriptScanner::new();
    let lexer = Lexer::new(scanner, text, Lf);

    let res = one(AtmaToken::CommandChunk)(lexer.clone());

    let actual = format!("{}", res.err().unwrap());
    let expected = "\
error: unexpected end of text
 --> [SOURCE TEXT]:0:0 (byte 0)
  | 
0 | 
  | ^ text ends here
\
".to_string();

    println!("{}", actual);
    println!("{}", expected);
    assert_eq!(actual, expected);
}

