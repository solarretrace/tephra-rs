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



pub fn interpolate_range_from_ast_expr<'text, Cm>(
    ast: AstExpr<'text>,
    metrics: Cm)
    -> Result<InterpolateRange, ParseError<'text, Cm>>
    where Cm: ColumnMetrics,
{
    match Ident::match_expr(ast.clone(), metrics) {
        Ok(Ident(ident)) if ident == "linear" => return Ok(
            InterpolateRange {
                interpolate_fn: InterpolateFunction::Linear,
                .. Default::default()
            }
        ),
        Ok(Ident(ident)) if ident == "cubic" => return Ok(
            InterpolateRange {
                interpolate_fn: InterpolateFunction::Cubic(0.0, 1.0),
                .. Default::default()
            }
        ),
        _ => (),
    }


    let AstExpr::Unary(Spanned { span, value }) = ast;
    let ast_span = span;
    match value {
        UnaryExpr::Call(CallExpr::Call { target, args }) => {
            let mut res = InterpolateRange::default();

            // Check "cubic" and "linear" targets.
            match Ident::match_primary_expr(
                target.value.clone(),
                target.span,
                metrics)
            {
                Ok(Ident(i)) if i != "linear" || i != "cubic" => {
                    return Err(ParseError::new("expected interpolation function")
                        .with_span("expected 'linear' or 'cubic'",
                            target.span,
                            metrics));
                },
                Ok(Ident(i)) if i == "cubic" => {
                    res.interpolate_fn = InterpolateFunction::Cubic(0.0, 1.0);
                }
                _ => (),
            }

            match <FunctionCall<Ident, (f32, f32)>>::match_primary_expr(
                target.value.clone(),
                target.span,
                metrics)
            {
                Ok(FunctionCall { target: Ident(i), args }) if i == "cubic" => {
                    valid_unit_range(args.0, args.1)
                        .map_err(|e| e.with_span(
                            "invalid interpolation parameters",
                            ast_span,
                            metrics))?;
                    res.interpolate_fn = InterpolateFunction::Cubic(
                        args.0,
                        args.1);
                },
                _ => return Err(ParseError::new(
                        "expected interpolation function")
                    .with_span("unrecognized interpolation function",
                        target.span,
                        metrics)),
            }

            match <(Vec<f32>,)>::match_primary_expr(
                PrimaryExpr::Tuple(args.clone()),
                ast_span,
                metrics)
            {
                Ok((args,)) if args.len() != 2 => return Err(
                    ParseError::new(
                        "expected [f32, f32] for interpolation range")
                        .with_span("invalid range value",
                            target.span,
                            metrics)),
                Ok((args,)) => {
                    valid_unit_range(args[0], args[1])
                        .map_err(|e| e.with_span("invalid range value",
                            ast_span,
                            metrics))?;
                    res.start = args[0];
                    res.end = args[1];
                },
                _ => (),
            }

            match <(Vec<f32>, Ident)>::match_primary_expr(
                PrimaryExpr::Tuple(args.clone()),
                ast_span,
                metrics)
            {
                Ok((args, _)) if args.len() != 2 => return Err(
                    ParseError::new(
                        "expected [f32, f32] for interpolation range")
                        .with_span("invalid range value",
                            target.span,
                            metrics)),
                Ok((_, Ident(i))) if i != "rgb" => return Err(
                    ParseError::new(
                        "expected color space keyword")
                    .with_span("unrecognized color space",
                        target.span,
                        metrics)),
                Ok((args, Ident(i))) => {
                    valid_unit_range(args[0], args[1])
                        .map_err(|e| e.with_span("invalid range value",
                            ast_span,
                            metrics))?;

                    res.start = args[0];
                    res.end = args[1];
                    match i.as_ref() {
                        "rgb" => res.color_space = ColorSpace::Rgb,
                        _ => unreachable!(),
                    }
                },
                _ => (),
            }

            match <(Ident, )>::match_primary_expr(
                PrimaryExpr::Tuple(args),
                ast_span,
                metrics)
            {
                Ok((Ident(i),)) if i != "rgb" => return Err(
                    ParseError::new(
                        "expected color space keyword")
                    .with_span("unrecognized color space",
                        target.span,
                        metrics)),

                Ok((Ident(i),)) => match i.as_ref() {
                    "rgb" => res.color_space = ColorSpace::Rgb,
                    _ => unreachable!(),
                },
                _ => (),
            }

            Ok(res)
        },

        _ => Err(ParseError::new("expected interpolation function")
            .with_span("unrecognized interpolation function",
                ast_span,
                metrics)),
        
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
