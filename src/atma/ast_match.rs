////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma AST expression matchers.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::*;
use crate::position::ColumnMetrics;
use crate::result::ParseError;
use crate::result::Spanned;

// External library imports.
use ::color::Color;
use ::color::Rgb;
use ::color::Xyz;
use ::color::Hsl;
use ::color::Hsv;
use ::color::Cmyk;

// Standard library imports.
use std::str::FromStr as _;

////////////////////////////////////////////////////////////////////////////////
// AstExprMatch
////////////////////////////////////////////////////////////////////////////////
pub trait AstExprMatch: Sized {
    fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics;
}


////////////////////////////////////////////////////////////////////////////////
// Ident matcher
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for String {
    fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new(
                concat!("invalid identifier"))
            .with_span(concat!("not a valid identifier"),
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Ident(ident))) => {
                Ok(ident.to_string())
            },

            _ => Err(default_error),
        }
    }    
}


////////////////////////////////////////////////////////////////////////////////
// Numeric matchers
////////////////////////////////////////////////////////////////////////////////

macro_rules! negatable_numeric_matcher {
    ($t:ty, $rep:expr) => {
        impl AstExprMatch for $t {
            fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("invalid ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { operand, .. } => {
                        <$t>::match_ast(AstExpr::Unary(*operand), metrics)
                            .map(|f| -f)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Float(float))) => {
                        <$t>::from_str(float)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("invalid ", $rep, " value"))
                            .with_span(concat!($rep, " is not callable"),
                                ast_span,
                                metrics)),

                    _ => Err(default_error),
                }
            }    
        }
    };
}

macro_rules! nonnegatable_numeric_matcher {
    ($t:ty, $rep:expr) => {
        impl AstExprMatch for $t {
            fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("invalid ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { .. } => Err(
                        ParseError::new(concat!("invalid ", $rep, " value"))
                            .with_span(concat!($rep, " is not negatable"),
                                ast_span,
                                metrics)),

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Float(float))) => {
                        <$t>::from_str(float)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("invalid ", $rep, " value"))
                            .with_span(concat!($rep, " is not callable"),
                                ast_span,
                                metrics)),

                    _ => Err(default_error),
                }
            }    
        }
    };
}

negatable_numeric_matcher!(f32, "f32");
negatable_numeric_matcher!(f64, "f64");

negatable_numeric_matcher!(i8, "i8");
negatable_numeric_matcher!(i16, "i16");
negatable_numeric_matcher!(i32, "i32");
negatable_numeric_matcher!(i64, "i64");
negatable_numeric_matcher!(isize, "isize");

nonnegatable_numeric_matcher!(u8, "u8");
nonnegatable_numeric_matcher!(u16, "u16");
nonnegatable_numeric_matcher!(u32, "u32");
nonnegatable_numeric_matcher!(u64, "u64");
nonnegatable_numeric_matcher!(usize, "usize");

////////////////////////////////////////////////////////////////////////////////
// Tuple matchers
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for () {
    fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Tuple(tuple)))
                if tuple.is_empty() => 
            {
                Ok(())
            },

            _ => Err(ParseError::new("invalid unit value")
                .with_span("not a valid unit value",
                    ast_span,
                    metrics)),
        }
    }    
}

macro_rules! tuple_impls {
    ($(
        $Tuple:ident { $($T:ident)+ }
    )+) => {
        $(
            impl<$($T:AstExprMatch),+> AstExprMatch for ($($T),+,) {
                fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                    -> Result<Self, ParseError<'text, Cm>>
                    where Cm: ColumnMetrics
                {
                    let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                    let ast_span = span;

                    match value {
                        UnaryExpr::Call(
                            CallExpr::Primary(
                                PrimaryExpr::Tuple(mut tuple))) =>
                        {
                            let res = ($({
                                if tuple.is_empty() {
                                    return Err(ParseError::new("invalid tuple value")
                                        .with_span("not a valid tuple value",
                                            ast_span,
                                            metrics));
                                };

                                <$T>::match_ast(tuple.remove(0), metrics)?
                                },
                            )+);

                            if tuple.is_empty() {
                                Ok(res)
                            } else {
                                Err(ParseError::new("invalid tuple value")
                                    .with_span("not a valid tuple value",
                                        ast_span,
                                        metrics))
                            }
                        },

                        _ => Err(ParseError::new("invalid tuple value")
                            .with_span("not a valid tuple value",
                                ast_span,
                                metrics)),
                    }
                }    
            }

        )+
    }
}

tuple_impls! {
    Tuple1 { A }
    Tuple2 { A B }
    Tuple3 { A B C }
    Tuple4 { A B C D }
    Tuple5 { A B C D E }
    Tuple6 { A B C D E F }
    Tuple7 { A B C D E F G }
    Tuple8 { A B C D E F G H }
    Tuple9 { A B C D E F G H I }
    Tuple10 { A B C D E F G H I J }
    Tuple11 { A B C D E F G H I J K }
    Tuple12 { A B C D E F G H I J K L }
}

////////////////////////////////////////////////////////////////////////////////
// Color matcher
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for Color {
    fn match_ast<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new(
                concat!("invalid color"))
            .with_span(concat!("not a valid color"),
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Color(color))) => {
                Ok(color)
            },

            UnaryExpr::Call(CallExpr::Call { target, args }) => {
                let target = AstExpr::Unary(Spanned { 
                    value: UnaryExpr::Call(CallExpr::Primary(target.value)),
                    span: target.span,
                });
                let args = AstExpr::Unary(Spanned { 
                    value: UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Tuple(args))),
                    span: ast_span,
                });

                let target = String::match_ast(target, metrics)?;
                match target.as_ref() {
                    "rgb"  => {
                        let (r, g, b) = <(f32, f32, f32)>::match_ast(
                            args,
                            metrics)?;
                        Ok(Color::from(Rgb::from([r, g, b])))
                    },
                    "xzy"  => {
                        let (x, y, z) = <(f32, f32, f32)>::match_ast(
                            args,
                            metrics)?;
                        Ok(Color::from(Xyz::from([x, y, z])))
                    },
                    "hsl"  => {
                        let (h, s, l) = <(f32, f32, f32)>::match_ast(
                            args,
                            metrics)?;
                        Ok(Color::from(Hsl::from([h, s, l])))
                    },
                    "hsv"  => {
                        let (h, s, v) = <(f32, f32, f32)>::match_ast(
                            args,
                            metrics)?;
                        Ok(Color::from(Hsv::from([h, s, v])))
                    },
                    "cmyk" => {
                        let (c, m, y, k) = <(f32, f32, f32, f32)>::match_ast(
                            args,
                            metrics)?;
                        Ok(Color::from(Cmyk::from([c, m, y, k])))
                    },
                    _      => Err(default_error)
                }
            }

            _ => Err(default_error),
        }
    }    
}
