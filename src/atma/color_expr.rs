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

// Standard library imports.
use std::str::FromStr as _;

////////////////////////////////////////////////////////////////////////////////
// RampExpr
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for RampExpr {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        
        match <FunctionCall<Ident, (u8, BlendFunction)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand: Ident(i), args }) if i == "ramp" => {
                return Ok(RampExpr {
                    count: args.0,
                    blend_fn: args.1,
                    interpolate: InterpolateRange::default(),
                });
            },
            _ => (),
        }

        match <FunctionCall<Ident, (
                u8,
                BlendFunction,
                InterpolateRange)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand: Ident(i), args }) if i == "ramp" => {
                return Ok(RampExpr {
                    count: args.0,
                    blend_fn: args.1,
                    interpolate: args.2,
                });
            },
            _ => (),
        }

        Err(ParseError::new("invalid ramp function")
            .with_span("unrecognized ramp function",
                ast_span,
                metrics))
    }
}


////////////////////////////////////////////////////////////////////////////////
// BlendExpr
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for BlendExpr {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        
        match <FunctionCall<UnaryBlendMethod, (
                CellRef<'static>,
                f32)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(BlendExpr {
                    blend_fn: BlendFunction::Unary(UnaryBlendFunction {
                        blend_method: operand,
                        value: args.1,
                        arg: args.0,
                    }),
                    interpolate: Interpolate::default(),
                });
            },
            _ => (),
        }

        match <FunctionCall<UnaryBlendMethod, (
                CellRef<'static>,
                f32,
                Interpolate)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(BlendExpr {
                    blend_fn: BlendFunction::Unary(UnaryBlendFunction {
                        blend_method: operand,
                        value: args.1,
                        arg: args.0,
                    }),
                    interpolate: args.2,
                });
            },
            _ => (),
        }

        match <FunctionCall<BinaryBlendMethod, (
                CellRef<'static>,
                CellRef<'static>)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(BlendExpr {
                    blend_fn: BlendFunction::Binary(BinaryBlendFunction {
                        blend_method: operand,
                        color_space: ColorSpace::Rgb,
                        arg_0: args.0,
                        arg_1: args.1,
                    }),
                    interpolate: Interpolate::default(),
                });
            },
            _ => (),
        }

        match <FunctionCall<BinaryBlendMethod, (
                CellRef<'static>,
                CellRef<'static>,
                Interpolate)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(BlendExpr {
                    blend_fn: BlendFunction::Binary(BinaryBlendFunction {
                        blend_method: operand,
                        color_space: ColorSpace::Rgb,
                        arg_0: args.0,
                        arg_1: args.1,
                    }),
                    interpolate: args.2,
                });
            },
            _ => (),
        }

        match <FunctionCall<BinaryBlendMethod, (
                CellRef<'static>,
                CellRef<'static>,
                Interpolate,
                ColorSpace)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(BlendExpr {
                    blend_fn: BlendFunction::Binary(BinaryBlendFunction {
                        blend_method: operand,
                        color_space: args.3,
                        arg_0: args.0,
                        arg_1: args.1,
                    }),
                    interpolate: args.2,
                });
            },
            _ => (),
        }

        match <FunctionCall<BinaryBlendMethod, (
                CellRef<'static>,
                CellRef<'static>,
                ColorSpace)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                
                return Ok(BlendExpr {
                    blend_fn: BlendFunction::Binary(BinaryBlendFunction {
                        blend_method: operand,
                        color_space: args.2,
                        arg_0: args.0,
                        arg_1: args.1,
                    }),
                    interpolate: Interpolate::default(),
                });
            },
            _ => (),
        }

        Err(ParseError::new("invalid blend function")
            .with_span("unrecognized blend function",
                ast_span,
                metrics))
    }
}

////////////////////////////////////////////////////////////////////////////////
// BlendFunction
////////////////////////////////////////////////////////////////////////////////
impl AstExprMatch for BlendFunction {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();

        // Unary
        match <UnaryBlendFunction>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(unary) => return Ok(BlendFunction::Unary(unary)),
            _ => (),
        }

        // Binary
        match <BinaryBlendFunction>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(binary) => return Ok(BlendFunction::Binary(binary)),
            _ => (),
        }

        Err(ParseError::new("invalid blend function")
            .with_span("unrecognized blend function",
                ast_span,
                metrics))
    }
}

impl AstExprMatch for UnaryBlendFunction {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();

        match <FunctionCall<
                UnaryBlendMethod,
                (f32, CellRef<'static>)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(UnaryBlendFunction {
                    blend_method: operand,
                    value: args.0,
                    arg: args.1,
                });
            },
            _ => (),
        }

        Err(ParseError::new("invalid blend function")
            .with_span("unrecognized blend function",
                ast_span,
                metrics))
    }
}


impl AstExprMatch for BinaryBlendFunction {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {

        let ast_span = ast_expr.span();

        match <FunctionCall<
                BinaryBlendMethod,
                (ColorSpace, CellRef<'static>, CellRef<'static>)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                return Ok(BinaryBlendFunction {
                    blend_method: operand,
                    color_space: args.0,
                    arg_0: args.1,
                    arg_1: args.2,
                });
            },
            _ => (),
        }

        Err(ParseError::new("invalid blend function")
            .with_span("unrecognized blend function",
                ast_span,
                metrics))
    }
}




////////////////////////////////////////////////////////////////////////////////
// BlendMethod
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for UnaryBlendMethod {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        
        match Ident::match_expr(ast_expr, metrics) {
            Ok(Ident(i)) => match UnaryBlendMethod::from_str(i.as_ref()) {
                Ok(blend) => return Ok(blend),
                Err(_)    => (),
            },
            _ => (),
        }

        Err(ParseError::new("invalid blend function")
            .with_span("unrecognized blend function",
                ast_span,
                metrics))
    }
}

impl AstExprMatch for BinaryBlendMethod {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();
        
        match Ident::match_expr(ast_expr, metrics) {
            Ok(Ident(i)) => match BinaryBlendMethod::from_str(i.as_ref()) {
                Ok(blend) => return Ok(blend),
                Err(_)    => (),
            },
            _ => (),
        }

        Err(ParseError::new("invalid blend function")
            .with_span("unrecognized blend function",
                ast_span,
                metrics))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Interpolate
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for Interpolate {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let ast_span = ast_expr.span();

        match f32::match_expr(ast_expr.clone(), metrics) {
            Ok(amount) => {
                if amount > 1.0 || amount < 0.0 {
                    return Err(ParseError::new("invalid interpolate value")
                        .with_span("value must lie in the range [0.0, 1.0]",
                            ast_span,
                            metrics));
                }
                return Ok(Interpolate {
                    amount,
                    .. Default::default()
                });
            },
            _ => (),
        }

        match <FunctionCall<InterpolateFunction, (f32,)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                if args.0 > 1.0 || args.0 < 0.0 {
                    return Err(ParseError::new("invalid interpolate value")
                        .with_span("value must lie in the range [0.0, 1.0]",
                            ast_span,
                            metrics));
                }
                return Ok(Interpolate {
                    interpolate_fn: operand,
                    amount: args.0,
                    .. Default::default()
                });
            },
            _ => (),
        }

        match <FunctionCall<InterpolateFunction, (f32, ColorSpace)>>::match_expr(
            ast_expr.clone(),
            metrics)
        {
            Ok(FunctionCall { operand, args }) => {
                if args.0 > 1.0 || args.0 < 0.0 {
                    return Err(ParseError::new("invalid interpolate value")
                        .with_span("value must lie in the range [0.0, 1.0]",
                            ast_span,
                            metrics));
                }
                return Ok(Interpolate {
                    interpolate_fn: operand,
                    amount: args.0,
                    color_space: args.1,
                    .. Default::default()
                });
            },
            _ => (),
        }

        Err(ParseError::new("expected interpolate value")
            .with_span("unrecognized interpolate value", ast_span, metrics))
    }
}

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
            ast_expr,
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

        Err(ParseError::new("expected interpolate range")
            .with_span("unrecognized interpolate range", ast_span, metrics))
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

        Err(ParseError::new("expected interpolate function")
            .with_span("unrecognized interpolate function",
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
