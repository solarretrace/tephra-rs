////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser combinators for the Atma AST.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]

// Local imports.
use crate::atma::*;
use crate::atma::cell_ref;
use crate::combinator::atomic;
use crate::combinator::both;
use crate::combinator::bracket;
use crate::combinator::fail;
use crate::combinator::intersperse_collect;
use crate::combinator::repeat_collect;
use crate::combinator::one;
use crate::combinator::section;
use crate::combinator::spanned;
use crate::combinator::text;
use crate::lexer::Lexer;
use crate::position::ColumnMetrics;
use crate::result::Failure;
use crate::result::ParseError;
use crate::result::Success;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::span::Span;
use crate::result::Spanned;

// Standard library imports.
use std::borrow::Cow;

// External library imports.
use ::color::Color;

////////////////////////////////////////////////////////////////////////////////
// AstExpr
////////////////////////////////////////////////////////////////////////////////

/// The top-level AST expression. Has the lowest precedence.
#[derive(Debug, Clone, PartialEq)]
pub enum AstExpr<'text> {
    /// A unary expression.  Defer to higher precedence operators.
    Unary(Spanned<'text, UnaryExpr<'text>>),
}

impl<'text> AstExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use AstExpr::*;
        match self {
            Unary(_) => "expression".into(),
        }
    }

    pub fn span(&self) -> Span<'text> {
        match self {
            AstExpr::Unary(Spanned { span, .. }) => *span,
        }
    }
}

/// A unary AST expression. Has lower precedence than CallExpr.
#[derive(Debug, Clone, PartialEq)]
#[allow(variant_size_differences)]
pub enum UnaryExpr<'text> {
    /// A numerical negation expression.
    Minus {
        op: Span<'text>,
        operand: Box<Spanned<'text, UnaryExpr<'text>>>,
    },
    /// A function call expression. Defer to higher precedence operators.
    Call(CallExpr<'text>),
}

impl<'text> UnaryExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use UnaryExpr::*;
        match self {
            Minus { operand, .. } => {
                format!("negated {}", operand.value.description()).into()
            },
            Call(p) => p.description()
        }
    }
}

/// A function call AST expression. Has lower precedence than PrimaryExpr.
#[derive(Debug, Clone, PartialEq)]
pub enum CallExpr<'text> {
    /// A function call expression.
    Call {
        operand: Box<Spanned<'text, CallExpr<'text>>>,
        args: Vec<AstExpr<'text>>,
    },
    /// A primary expression. Defer to higher precedence operators.
    Primary(PrimaryExpr<'text>),
}

impl<'text> CallExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use CallExpr::*;
        match self {
            Call { .. } => "function call".into(),
            Primary(p)  => p.description()
        }
    }
}

/// A primitive or grouped AST expression. Has the highest precedence.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimaryExpr<'text> {
    /// An identifier.
    Ident(&'text str),
    /// An integral value.
    Uint(&'text str),
    /// A floating point value.
    Float(&'text str),
    /// A Color value.
    Color(Color),
    /// A CellRef value.
    CellRef(CellRef<'text>),
    /// A bracketted group of values.
    Array(Vec<AstExpr<'text>>),
    /// A parenthesized group of values.
    Tuple(Vec<AstExpr<'text>>),
}

impl<'text> PrimaryExpr<'text> {
    pub fn description(&self) -> Cow<'static, str> {
        use PrimaryExpr::*;
        match self {
            Ident(_)     => "identifier".into(),
            Uint(_)      => "integer value".into(),
            Float(_)     => "float value".into(),
            Color(_)     => "color value".into(),
            CellRef(_)   => "cell reference".into(),
            Array(elems) => format!("{} element array", elems.len()).into(),
            Tuple(elems) => format!("{} element tuple", elems.len()).into(),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Parsers
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
        Some(Minus) => both(
                spanned(one(Minus)),
                spanned(unary_expr))
            (lexer)
            .map_value(|(op, u)| UnaryExpr::Minus {
                op: op.span,
                operand: Box::new(u)
            }),

        Some(_) => call_expr
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

    let (Spanned { value, mut span }, mut succ) = spanned(primary_expr)
        (lexer)?
        .take_value();

    let mut res = CallExpr::Primary(value);

    loop {
        match atomic(
                spanned(bracket(
                    one(OpenParen),
                    intersperse_collect(0, None,
                        ast_expr,
                        one(Comma)),
                    one(CloseParen))))
            (succ.lexer.clone())
            .filter_lexer_error()
        {
            Ok(args_succ) => match args_succ.value {
                Some(Spanned { value: args, span: args_span }) => {
                    res = CallExpr::Call {
                        operand: Box::new(Spanned {
                            value: res,
                            span,
                        }),
                        args,
                    };
                    span = span.enclose(args_span);
                    succ.lexer = args_succ.lexer;
                },
                None => break,
            },
            Err(None)     => break,
            Err(Some(e))  => return Err(e),
        }
    }

    Ok(Success {
        value: res,
        lexer: succ.lexer,
    })
}

pub fn primary_expr<'text, Cm>(lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, PrimaryExpr<'text>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    match lexer.peek() {
        Some(Ident) => text(one(Ident))
            (lexer)
            .map_value(PrimaryExpr::Ident),

        Some(Float) => text(one(Float))
            (lexer)
            .map_value(PrimaryExpr::Float),

        Some(Uint) => text(one(Uint))
            (lexer)
            .map_value(PrimaryExpr::Uint),

        Some(OpenParen) => bracket(
                one(OpenParen),
                intersperse_collect(0, None,
                    section(ast_expr),
                    one(Comma)),
                one(CloseParen))
            (lexer)
            .map_value(PrimaryExpr::Tuple),

        Some(OpenBracket) => bracket(
                one(OpenBracket),
                intersperse_collect(0, None,
                    section(ast_expr),
                    one(Comma)),
                one(CloseBracket))
            (lexer)
            .map_value(PrimaryExpr::Array),
        
        Some(Hash) => color
            (lexer)
            .map_value(PrimaryExpr::Color),

        Some(Colon)             |
        Some(Mult)              |
        Some(StringOpenSingle)  |
        Some(StringOpenDouble)  |
        Some(RawStringOpen)     => cell_ref
            (lexer)
            .map_value(PrimaryExpr::CellRef),

        _ => fail
            (lexer)
            .map_value(|_| unreachable!())
    }
}
