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
// 
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


////////////////////////////////////////////////////////////////////////////////
// InterpolateRange
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for InterpolateRange {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        match InterpolateFunction::match_expr(ast_expr.clone(), metrics) {
            Ok(interpolate_fn) => {
                return Ok(InterpolateRange {
                    interpolate_fn,
                    .. Default::default()
                });
            },
            _ => (),
        }

        match <FunctionCall<InterpolateFunction, (Vec<f32>,)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) if args.0.len() != 2 => {
                return Err(ParseError::new("expected [f32, f32] value")
                    .with_span("wrong number of arguments", ast_span, metrics));
            },
            Ok(FunctionCall { operand, args }) => {
                valid_unit_range(args.0[0], args.0[1])
                        .map_err(|e| e.with_span("invalid range value",
                            ast_span,
                            metrics))?;
                return Ok(InterpolateRange {
                    interpolate_fn: operand,
                    start: args.0[0],
                    end: args.0[1],
                    .. Default::default()
                });
            },
            _ => (),
        }

        match <FunctionCall<InterpolateFunction, (Vec<f32>, ColorSpace)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) if args.0.len() != 2 => {
                return Err(ParseError::new("expected [f32, f32] value")
                    .with_span("wrong number of arguments", ast_span, metrics));
            },
            Ok(FunctionCall { operand, args }) => {
                valid_unit_range(args.0[0], args.0[1])
                        .map_err(|e| e.with_span("invalid range value",
                            ast_span,
                            metrics))?;
                return Ok(InterpolateRange {
                    interpolate_fn: operand,
                    start: args.0[0],
                    end: args.0[1],
                    color_space: args.1,
                    .. Default::default()
                });
            },
            _ => (),
        }

        match <FunctionCall<InterpolateFunction, (ColorSpace,)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(InterpolateRange {
                    color_space: args.0,
                    interpolate_fn: operand,
                    .. Default::default()
                });
            },
            _ => (),
        }

        Err(ParseError::new("expected interpolation range")
            .with_span("unrecognized interpolation range", ast_span, metrics))
    }
}

fn valid_unit_range<'text, Cm>(l: f32, r: f32)
    -> Result<(), ParseError<'text, Cm>>
    where Cm: ColumnMetrics,
{
    if l < 0.0 || l > 1.0 || r < 0.0 || r > 1.0 || r < l {
        Err(ParseError::new("value must lie in the range [0.0, 1.0]"))
    } else {
        Ok(())
    }
}


////////////////////////////////////////////////////////////////////////////////
// InterpolateFunction
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for InterpolateFunction {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        match Ident::match_expr(ast_expr.clone(), metrics) {
            Ok(Ident(ident)) if ident == "linear" => return Ok(
                InterpolateFunction::Linear
            ),
            Ok(Ident(ident)) if ident == "cubic" => return Ok(
                InterpolateFunction::Cubic(0.0, 0.0)
            ),
            _ => (),
        }

        match <FunctionCall<Ident, (f32, f32)>>::match_expr(ast_expr, metrics) {
            Ok(FunctionCall { operand: Ident(i), args }) if i == "cubic" => {
                return Ok(InterpolateFunction::Cubic(args.0, args.1));
            },
            _ => (),
        }

        Err(ParseError::new("expected interpolation function")
            .with_span("unrecognized interpolation function",
                ast_span,
                metrics))
    }
}

////////////////////////////////////////////////////////////////////////////////
// ColorSpace
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for ColorSpace {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        match Ident::match_expr(ast_expr, metrics) {
            Ok(Ident(ident)) if ident == "rgb" => Ok(ColorSpace::Rgb),

            _ => Err(ParseError::new("expected color space")
            .with_span("unrecognized color space", ast_span, metrics))
        }        
    }
}
