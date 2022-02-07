////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error formatting tests.
////////////////////////////////////////////////////////////////////////////////
// NOTE: Run the following command to get tracing output:
// RUST_LOG=TRACE cargo test --features "trace no-color" -- --show-output > .trace


// Internal library imports.
use crate::ParseError;
use crate::ParseErrorOwned;

// External library imports.
use pretty_assertions::assert_eq;
use tephra_span::ColumnMetrics;
use tephra_span::Pos;
use tephra_span::SourceText;
use tephra_span::Span;
use test_log::test;




/// Test basic ParseError formatting.
#[test]
#[tracing::instrument]
fn spanless_error() {
    colored::control::set_override(false);

    let e = ParseError::from("DESCRIPTION");

    let actual = format!("{e}");
    let expected = "error: DESCRIPTION\n";

    assert_eq!(actual, expected);
}


/// Test ParseError formatting for a spanned error over a single line.
#[test]
#[tracing::instrument]
fn single_line_spanned_error() {
    colored::control::set_override(false);

    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);
    let span = Span::new_enclosing(
        Pos::new(6, 4, 0),
        Pos::new(10, 4, 4));

    let e = ParseError::unrecognized_token(source, span);

    let actual = format!("{e}");
    let expected = "error: unrecognized token
 --> (4:0-4:4, bytes 6-10)
  | 
4 | abcd
  | ^^^^ symbol not recognized
";
    
    assert_eq!(actual, expected);
}

/// Test ParseError formatting for a spanned error over multiple lines.
#[test]
#[tracing::instrument]
fn multi_line_spanned_error() {
    colored::control::set_override(false);

    const TEXT: &'static str = "\n \n\n \nabcd\n def \nghi\n";
    let source = SourceText::new(TEXT);
    let span = Span::new_enclosing(
        Pos::new(3, 2, 0),
        Pos::new(10, 4, 4));

    let e = ParseError::unrecognized_token(source, span);

    let actual = format!("{e}");
    let expected = "error: unrecognized token
 --> (2:0-4:4, bytes 3-10)
  | 
2 | / 
3 | |  
4 | | abcd
  | |____^ symbol not recognized
";
    
    assert_eq!(actual, expected);
}
