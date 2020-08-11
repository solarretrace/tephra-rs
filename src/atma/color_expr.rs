////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for the Atma color expressions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::combinator::atomic;
use crate::combinator::both;
use crate::combinator::right;
use crate::combinator::one;
use crate::combinator::text;
use crate::combinator::bracket;
use crate::combinator::fail;
use crate::combinator::section;
use crate::combinator::intersperse_collect;
use crate::atma::cell_ref;
use crate::lexer::Lexer;
use crate::result::ParseError;
use crate::result::ParseResult;
use crate::result::Failure;
use crate::result::ParseResultExt as _;
use crate::position::ColumnMetrics;


////////////////////////////////////////////////////////////////////////////////
// primary_expr
////////////////////////////////////////////////////////////////////////////////

pub fn expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Expr>
    where Cm: ColumnMetrics,
{
    unimplemented!()
}

pub fn insert_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, InsertExpr>
    where Cm: ColumnMetrics,
{
    unimplemented!()
}

pub fn blend_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, BlendExpr>
    where Cm: ColumnMetrics,
{
    unimplemented!()
}

pub fn blend_function<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, BlendFunction>
    where Cm: ColumnMetrics,
{
    unimplemented!()
}

