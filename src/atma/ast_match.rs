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
use crate::span::Span;

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
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics;

    fn match_unary_expr<'text, Cm>(
        unary_expr: UnaryExpr<'text>,
        span: Span<'text>,
        metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        Self::match_expr(
            AstExpr::Unary(Spanned { span, value: unary_expr }),
            metrics)
    }

    fn match_call_expr<'text, Cm>(
        call_expr: CallExpr<'text>,
        span: Span<'text>,
        metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        Self::match_unary_expr(
            UnaryExpr::Call(call_expr),
            span,
            metrics)
    }

    fn match_primary_expr<'text, Cm>(
        primary_expr: PrimaryExpr<'text>,
        span: Span<'text>,
        metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        Self::match_call_expr(
            CallExpr::Primary(primary_expr),
            span,
            metrics)
    }
}


////////////////////////////////////////////////////////////////////////////////
// Ident matcher
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

impl AstExprMatch for Ident {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new("expected identifier")
            .with_span("not a valid identifier",
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Ident(ident))) => {
                Ok(Ident(ident.to_string()))
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
            fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("expected ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { operand, .. } => {
                        <$t>::match_expr(AstExpr::Unary(*operand), metrics)
                            .map(|f| -f)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Float(float))) => {
                        <$t>::from_str(float)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
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
            fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
                -> Result<Self, ParseError<'text, Cm>>
                where Cm: ColumnMetrics
            {
                let AstExpr::Unary(Spanned { span, value }) = ast_expr;
                let ast_span = span;

                let default_error = ParseError::new(
                        concat!("expected ", $rep, " value"))
                    .with_span(concat!("not a valid ", $rep, " value"),
                        ast_span,
                        metrics);

                match value {
                    UnaryExpr::Minus { .. } => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
                            .with_span(concat!($rep, " is not negatable"),
                                ast_span,
                                metrics)),

                    UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Float(float))) => {
                        <$t>::from_str(float)
                            .map_err(|_| default_error)
                    },

                    UnaryExpr::Call(CallExpr::Call { .. }) => Err(
                        ParseError::new(concat!("expected ", $rep, " value"))
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
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
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

            _ => Err(ParseError::new("expected unit value")
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
                fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
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
                                    return Err(ParseError::new("expected tuple value")
                                        .with_span("not a valid tuple value",
                                            ast_span,
                                            metrics));
                                };

                                <$T>::match_expr(tuple.remove(0), metrics)?
                                },
                            )+);

                            if tuple.is_empty() {
                                Ok(res)
                            } else {
                                Err(ParseError::new("expected tuple value")
                                    .with_span("not a valid tuple value",
                                        ast_span,
                                        metrics))
                            }
                        },

                        _ => Err(ParseError::new("expected tuple value")
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
    Tuple1  { A }
    Tuple2  { A B }
    Tuple3  { A B C }
    Tuple4  { A B C D }
    Tuple5  { A B C D E }
    Tuple6  { A B C D E F }
    Tuple7  { A B C D E F G }
    Tuple8  { A B C D E F G H }
    Tuple9  { A B C D E F G H I }
    Tuple10 { A B C D E F G H I J }
    Tuple11 { A B C D E F G H I J K }
    Tuple12 { A B C D E F G H I J K L }
}



////////////////////////////////////////////////////////////////////////////////
// Array matcher
////////////////////////////////////////////////////////////////////////////////

// TODO: Attempting to initialize an array [T; N] butts up against the rules
// for safely initializing an array dynamically, which forbids a generic
// transmute of [MaybeUninit<T>; N] to [T; N]. I don't know of a good
// workaround, so we'll just use Vec instead and verify the length externally.
impl<T> AstExprMatch for Vec<T> where T: AstExprMatch {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        match value {
            UnaryExpr::Call(
                CallExpr::Primary(
                    PrimaryExpr::Array(array))) =>
            {
                let mut res = Vec::with_capacity(array.len());
                    
                for elem in array.into_iter() {
                    res.push(T::match_expr(elem, metrics)?);
                }

                Ok(res)
            },

            _ => Err(ParseError::new("expected array value")
                .with_span("not a valid array value",
                    ast_span,
                    metrics)),
        }
    }    
}


////////////////////////////////////////////////////////////////////////////////
// Color matcher
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for Color {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new(
                concat!("expected color"))
            .with_span(concat!("not a valid color"),
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::Color(color))) => {
                Ok(color)
            },

            UnaryExpr::Call(CallExpr::Call { operand, args }) => {
                let operand = Ident::match_call_expr(
                    operand.value,
                    operand.span,
                    metrics)?.0;
                match operand.as_ref() {
                    "rgb"  => {
                        let (r, g, b) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Rgb::from([r, g, b])))
                    },
                    "xzy"  => {
                        let (x, y, z) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Xyz::from([x, y, z])))
                    },
                    "hsl"  => {
                        let (h, s, l) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Hsl::from([h, s, l])))
                    },
                    "hsv"  => {
                        let (h, s, v) = <(f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
                            metrics)?;
                        Ok(Color::from(Hsv::from([h, s, v])))
                    },
                    "cmyk" => {
                        let (c, m, y, k) = <(f32, f32, f32, f32)>::match_primary_expr(
                            PrimaryExpr::Tuple(args),
                            ast_span,
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

////////////////////////////////////////////////////////////////////////////////
// CellRef matcher
////////////////////////////////////////////////////////////////////////////////

impl AstExprMatch for CellRef<'static> {
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new("expected cell reference")
            .with_span("not a valid cell reference",
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Primary(PrimaryExpr::CellRef(cell_ref))) => {
                Ok(cell_ref.into_static())
            },

            _ => Err(default_error),
        }
    }    
}



////////////////////////////////////////////////////////////////////////////////
// Ident matcher
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionCall<T, A> where T: AstExprMatch, A: AstExprMatch {
    pub operand: T,
    pub args: A,
}


impl<T, A> AstExprMatch for FunctionCall<T, A> 
    where T: AstExprMatch, A: AstExprMatch
{
    fn match_expr<'text, Cm>(ast_expr: AstExpr<'text>, metrics: Cm)
        -> Result<Self, ParseError<'text, Cm>>
        where Cm: ColumnMetrics
    {
        let AstExpr::Unary(Spanned { span, value }) = ast_expr;
        let ast_span = span;

        let default_error = ParseError::new("expected function call expression")
            .with_span("not a valid function call expression",
                ast_span,
                metrics);

        match value {
            UnaryExpr::Call(CallExpr::Call { operand, args }) => {
                let Spanned { span, value } = *operand;
                Ok(FunctionCall {
                    operand: T::match_call_expr(value, span, metrics)?,
                    args: A::match_primary_expr(
                        PrimaryExpr::Tuple(args),
                        ast_span,
                        metrics)?,
                })
            },

            _ => Err(default_error),
        }
    }    
}
