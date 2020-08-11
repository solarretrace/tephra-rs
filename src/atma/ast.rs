////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for the Atma AST.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::atma::cell_ref;
use crate::combinator::atomic;
use crate::combinator::both;
use crate::combinator::bracket;
use crate::combinator::fail;
use crate::combinator::intersperse_collect;
use crate::combinator::one;
use crate::combinator::right;
use crate::combinator::section;
use crate::combinator::spanned;
use crate::combinator::text;
use crate::lexer::Lexer;
use crate::position::ColumnMetrics;
use crate::result::Failure;
use crate::result::ParseError;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;


////////////////////////////////////////////////////////////////////////////////
// primary_expr
////////////////////////////////////////////////////////////////////////////////

pub fn ast_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, AstExpr<'text>>
    where Cm: ColumnMetrics,
{
    spanned(unary_expr)
        (lexer)
        .map_value(AstExpr::Unary)
}

pub fn unary_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, UnaryExpr<'text>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    match lexer.peek() {
        Some(Minus) => right(
                one(Minus),
                spanned(unary_expr))
            (lexer)
            .map_value(|u| UnaryExpr::Minus(Box::new(u))),

        Some(_) => spanned(call_expr)
            (lexer)
            .map_value(UnaryExpr::Call),

        None => Err(Failure {
            parse_error: ParseError::unexpected_end_of_text(
                lexer.end_span(),
                lexer.column_metrics()),
            lexer,
            source: None,
        }),
    }
}

pub fn call_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, CallExpr<'text>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    both(
        spanned(primary_expr),
        atomic(
            bracket(
                one(OpenParen),
                spanned(intersperse_collect(0, None,
                    section(spanned(ast_expr)),
                    one(Comma))),
                one(CloseParen))))
        (lexer)
        .map_value(|(l, r)| match r {
            Some(args) => CallExpr::Call(l, args),
            None       => CallExpr::Primary(l),
        })
}

pub fn primary_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, PrimaryExpr<'text>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    match lexer.peek() {
        Some(Ident) => spanned(text(one(Ident)))
            (lexer)
            .map_value(PrimaryExpr::Ident),

        Some(Float) => spanned(text(one(Float)))
            (lexer)
            .map_value(PrimaryExpr::Float),

        Some(Uint) => spanned(text(one(Uint)))
            (lexer)
            .map_value(PrimaryExpr::Uint),

        Some(OpenParen) => bracket(
                one(OpenParen),
                spanned(intersperse_collect(0, None,
                    section(spanned(ast_expr)),
                    one(Comma))),
                one(CloseParen))
            (lexer)
            .map_value(PrimaryExpr::Tuple),

        Some(OpenBracket) => bracket(
                one(OpenBracket),
                spanned(intersperse_collect(0, None,
                    section(spanned(ast_expr)),
                    one(Comma))),
                one(CloseBracket))
            (lexer)
            .map_value(PrimaryExpr::Array),
        
        Some(Hash) => spanned(color)
            (lexer)
            .map_value(PrimaryExpr::Color),

        Some(Colon)             |
        Some(Mult)              |
        Some(StringOpenSingle)  |
        Some(StringOpenDouble)  |
        Some(RawStringOpen)     => spanned(cell_ref)
            (lexer)
            .map_value(PrimaryExpr::CellRef),

        _ => fail
            (lexer)
            .map_value(|_| unreachable!())
    }
}
