////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for the Atma color expressions.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

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
use crate::result::Spanned;
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

// pub fn blend_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
//     -> ParseResult<'text, AtmaScanner, Cm, BlendExpr>
//     where Cm: ColumnMetrics,
// {
//     unimplemented!()
// }

// pub fn blend_function<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
//     -> ParseResult<'text, AtmaScanner, Cm, BlendFunction>
//     where Cm: ColumnMetrics,
// {
//     unimplemented!()
// }



// pub fn interpolate_range_from_ast<'text, Cm>(ast: AstExpr<'text>, metrics: Cm)
//     -> Result<InterpolateRange, ParseError<'text, Cm>>
//     where Cm: ColumnMetrics,
// {
//     let AstExpr::Unary(Spanned { span, value }) = ast;
//     let ast_span = span;
//     match value {
//         UnaryExpr::Minus { op, .. } => Err(
//             ParseError::new("invalid interpolation function")
//                 .with_span("interpolation function cannot be negated",
//                     op,
//                     metrics)),

//         UnaryExpr::Call(CallExpr::Call { target, args }) => unimplemented!(),

//         UnaryExpr::Call(CallExpr::Primary(primary)) => match primary {
//             PrimaryExpr::Ident(ident) if ident == "linear" => Ok(
//                 InterpolateRange {
//                     interpolate_fn: InterpolateFunction::Linear,
//                     .. Default::default()
//                 }
//             ),
//             PrimaryExpr::Ident(ident) if ident == "cubic" => Ok(
//                 InterpolateRange {
//                     interpolate_fn: InterpolateFunction::Cubic(0.0, 1.0),
//                     .. Default::default()
//                 }
//             ),
//             _ => Err(ParseError::new("invalid interpolation function")
//                 .with_span("expected 'linear' or 'cubic'",
//                     ast_span,
//                     metrics)),
//         }
//     }
// }

// pub fn unit_range_from_ast<'text, Cm>(ast: AstExpr<'text>, metrics: Cm)
//     -> Result<(f32, f32), ParseError<'text, Cm>>
//     where Cm: ColumnMetrics,
// {
//     let AstExpr::Unary(Spanned { span, value }) = ast;
//     let ast_span = span;


//     match value {
//         UnaryExpr::Minus { op, .. } => Err(
//             ParseError::new("invalid range")
//                 .with_span("range cannot be negated",
//                     op,
//                     metrics)),

//         UnaryExpr::Call(CallExpr::Call { target, args }) => Err(
//             ParseError::new("invalid range")
//                 .with_span("range cannot be called",
//                     ast_span,
//                     metrics)),

//         UnaryExpr::Call(CallExpr::Primary(primary)) => match primary {
//             PrimaryExpr::Tuple(tuple) => unimplemented!(),
//             _ => Err(
//                 ParseError::new("invalid range")
//                     .with_span("expected tuple",
//                         ast_span,
//                         metrics)),
// }
