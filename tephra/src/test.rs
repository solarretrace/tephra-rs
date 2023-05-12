////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Tephra test module.
////////////////////////////////////////////////////////////////////////////////


use crate::Lexer;
use crate::Pos;
use crate::SourceTextRef;
use crate::SourceTextOwned;
use crate::Scanner;
use crate::Success;
use crate::ParseResult;
use tephra_span::Span;
use tephra_span::ColumnMetrics;
use tephra_span::LineEnding;


////////////////////////////////////////////////////////////////////////////////
// Void Scanner
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
struct Void;

impl std::fmt::Display for Void {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Scanner for Void {
    type Token = Void;

    fn scan(&mut self, _: SourceTextRef<'_>, _: Pos)
        -> Option<(Self::Token, Pos)>
    {
        None
    }
}

////////////////////////////////////////////////////////////////////////////////
// Object size canaries.
////////////////////////////////////////////////////////////////////////////////

#[test]
fn verify_lexer_size() {
    assert_eq!(std::mem::size_of::<Lexer<'_, Void>>(), 224);
}

#[test]
fn verify_pos_size() {
    assert_eq!(std::mem::size_of::<Pos>(), 24);
}

#[test]
fn verify_span_size() {
    assert_eq!(std::mem::size_of::<Span>(), 48);
}

#[test]
fn verify_success_size() {
    assert_eq!(std::mem::size_of::<Success<'_, Void, ()>>(), 224);
}

#[test]
fn verify_result_size() {
    assert_eq!(std::mem::size_of::<ParseResult<'_, Void, ()>>(), 224);
}

#[test]
fn verify_line_ending_size() {
    assert_eq!(std::mem::size_of::<LineEnding>(), 1);
}

#[test]
fn verify_column_metrics_size() {
    assert_eq!(std::mem::size_of::<ColumnMetrics>(), 2);
}

#[test]
fn verify_source_text_ref_size() {
    assert_eq!(std::mem::size_of::<SourceTextRef<'_>>(), 64);
}

#[test]
fn verify_source_text_owned_size() {
    assert_eq!(std::mem::size_of::<SourceTextOwned>(), 64);
}
