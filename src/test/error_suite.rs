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
use crate::result::Reason;
use crate::span::Span;
use crate::test::atma_script::*;


/// Tests `Failure` state for an unexpected empty parse.
#[test]
fn error_parse_one_from_empty() {
    let text = "";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);

    let res = one(AtmaToken::CommandChunk)(lexer.clone());

    assert_eq!(
        res,
        Err(Failure {
            lexer: lexer,
            span: Span::new(&text),
            reason: Reason::UnexpectedEndOfText,
            source: None,
        }));
}

/// Tests `Failure` message for an unexpected empty parse.
#[test]
fn error_empty_msg() {
    let text = "";
    let as_tok = AtmaScriptScanner::new();
    let lexer = Lexer::new(as_tok, text);

    let res = one(AtmaToken::CommandChunk)(lexer.clone());

    let actual = format!("{}", res.err().unwrap());
    let expected = "\
error: Unexpected end of text
  --> [text]:0:0 (byte 0)
   |
LN | [suround] [SPAN OF TOKEN] [suround]
   |            ~~~~~~~~~~~~~~~
... During parse of [CONTEXT].
\
".to_string();

    println!("{}", actual);
    println!("{}", expected);
    assert_eq!(actual, expected);
}

