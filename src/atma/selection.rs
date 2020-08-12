////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Cell reference and selection parsing.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]
    

// Local imports.
use crate::atma::*;
use crate::combinator::fail;
use crate::combinator::any;
use crate::combinator::atomic;
use crate::combinator::maybe;
use crate::combinator::both;
use crate::combinator::filter;
use crate::combinator::bracket;
use crate::combinator::bracket_dynamic;
use crate::combinator::exact;
use crate::combinator::repeat;
use crate::combinator::intersperse_collect;
use crate::combinator::left;
use crate::combinator::one;
use crate::combinator::right;
use crate::combinator::section;
use crate::combinator::seq;
use crate::combinator::text;
use crate::lexer::Lexer;
use crate::result::Failure;
use crate::result::Success;
use crate::result::ParseError;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::position::ColumnMetrics;

// Standard library imports.
use std::borrow::Cow;
use std::convert::TryFrom as _;


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

pub fn cell_ref<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, CellRef<'text>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    match lexer.peek() {
        Some(Colon) => position_or_index
            (lexer)
            .map_value(CellRef::from),

        Some(RawStringOpen)    |
        Some(StringOpenSingle) |
        Some(StringOpenDouble) => group_or_name
            (lexer)
            .map_value(|(name, idx)| match idx {
                Some(idx) => CellRef::Group { group: name, idx },
                None      => CellRef::Name(name),
            }),

        _ => fail
            (lexer)
            .map_value(|_| unreachable!())
    }
}

pub fn position_or_index<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, PositionOrIndex>
    where Cm: ColumnMetrics,
{
    lexer.filter_next();
    let (idx, idx_succ) = exact(
        right(one(AtmaToken::Colon),
            uint::<_, u32>))
        (lexer)?
        .take_value();

    match exact(
        atomic(
            both(
                right(one(AtmaToken::Decimal), uint::<_, u16>),
                right(one(AtmaToken::Decimal), uint::<_, u16>))))
        (idx_succ.lexer.clone())
    {
        Ok(succ) => {
            if succ.value.is_none() {
                Ok(Success {
                    value: PositionOrIndex::Index(idx),
                    lexer: idx_succ.lexer,
                })
            } else {
                let (line, column) = succ.value.unwrap();
                match u16::try_from(idx) {
                    Ok(page) => Ok(Success {
                        value: PositionOrIndex::Position(Position {
                            page, 
                            line,
                            column,
                        }),
                        lexer: succ.lexer,
                    }),
                    Err(e) => Err(Failure {
                        parse_error: ParseError::new("invalid integer value")
                            .with_span(format!(
                                "value ({}) is too large for unsigned 16 bit value",
                                    idx),
                                idx_succ.lexer.last_span(),
                                idx_succ.lexer.column_metrics()),
                        lexer: succ.lexer,
                        source: Some(Box::new(e)),
                    }),
                }
            }
        },
        Err(fail) => Err(fail),
    }
}

pub fn index<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, u32>
    where Cm: ColumnMetrics,
{
    lexer.filter_next();
    exact(
        right(one(AtmaToken::Colon),
            uint::<_, u32>))
        (lexer)
}

pub fn position<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Position>
    where Cm: ColumnMetrics,
{
    lexer.filter_next();
    exact(
        right(one(AtmaToken::Colon),
            both(
                uint::<_, u16>,
                both(
                    right(one(AtmaToken::Decimal), uint::<_, u16>),
                    right(one(AtmaToken::Decimal), uint::<_, u16>)))))
        (lexer)
        .map_value(|(page, (line, column))| Position { page, line, column })
}

pub fn group_or_name<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, (Cow<'text, str>, Option<u32>)>
    where Cm: ColumnMetrics,
{
    exact(
        both(
            string,
            atomic(
                right(one(AtmaToken::Colon),
                    uint::<_, u32>))))
        (lexer)
}

////////////////////////////////////////////////////////////////////////////////
// CellSelection
////////////////////////////////////////////////////////////////////////////////

pub fn cell_selection<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, CellSelection<'text>>
    where Cm: ColumnMetrics,
{
    intersperse_collect(1, None,
        section(cell_selector),
        one(AtmaToken::Comma))
        (lexer)
        .map_value(CellSelection)
}


////////////////////////////////////////////////////////////////////////////////
// CellSelector
////////////////////////////////////////////////////////////////////////////////

pub fn cell_selector<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, CellSelector<'text>>
    where Cm: ColumnMetrics,
{
    use AtmaToken::*;
    use CellSelector::*;
    match lexer.peek() {
        Some(Colon) => {
            match range(position_or_index)
                (lexer.clone())
                .filter_lexer_error()
            {
                Ok(succ) => {
                    let (val, succ) = succ.take_value();
                    use PositionOrIndex::*;
                    match val {
                        (Index(idx),    None) => return Ok(succ)
                            .map_value(|_| CellSelector::Index(idx)),

                        (Index(low),    Some(Index(high))) if low > high => {
                            return Err(Failure {
                                parse_error: ParseError::new("invalid index range")
                                    .with_span(
                                        "range bounds are in the wrong order", 
                                        succ.lexer.span(),
                                        succ.lexer.column_metrics()),
                                lexer: succ.lexer,
                                source: None,
                            })
                        },

                        (Index(low),    Some(Index(high))) => return Ok(succ)
                            .map_value(|_| IndexRange { low, high }),

                        (Position(pos), None) => return Ok(succ)
                            .map_value(|_| PositionSelector(pos.into())),

                        (Position(low), Some(Position(high))) if low > high => {
                            return Err(Failure {
                                parse_error: ParseError::new("invalid position range")
                                    .with_span(
                                        "range bounds are in the wrong order", 
                                        succ.lexer.span(),
                                        succ.lexer.column_metrics()),
                                lexer: succ.lexer,
                                source: None,
                            })
                        },

                        (Position(low), Some(Position(high))) => return Ok(succ)
                            .map_value(|_| PositionRange { low, high }),

                        _ => return Err(Failure {
                            parse_error: ParseError::new("invalid range")
                                .with_span(
                                    "range bounds have incompatable types", 
                                    succ.lexer.span(),
                                    succ.lexer.column_metrics()),
                            lexer: succ.lexer,
                            source: None,
                        }),
                    }
                },
                Err(Some(fail)) => return Err(fail),
                Err(None)       => (),
            }

            match position_selector
                (lexer.clone())
                .filter_lexer_error()
            {
                Ok(succ)        => return Ok(succ).map_value(PositionSelector),
                Err(Some(fail)) => return Err(fail),
                Err(None)       => (),
            }

            // All must come after PositionSelector. 
            exact(
                seq(&[Colon, Mult]))
                (lexer.clone())
                .map_value(|_| All)
        },

        Some(RawStringOpen)    |
        Some(StringOpenSingle) |
        Some(StringOpenDouble) => {
            match exact(
                left(
                    string,
                    seq(&[Colon, Mult])))
                (lexer.clone())
                .filter_lexer_error()
            {
                Ok(succ)        => return Ok(succ)
                    .map_value(|name| GroupAll(name)),
                Err(Some(fail)) => return Err(fail),
                Err(None)       => (),
            }

            // Group and Name must come after GroupAll.
            let (val, succ) = range(group_or_name)
                (lexer.clone())?
                .take_value();
            match val {
                ((l, Some(low)), Some((r, Some(high)))) if low > high => {
                    return Err(Failure {
                        parse_error: ParseError::new("invalid group range")
                            .with_span(
                                "range bounds are in the wrong order", 
                                succ.lexer.span(),
                                succ.lexer.column_metrics()),
                        lexer: succ.lexer,
                        source: None,
                    })
                },

                ((l, Some(low)), Some((r, Some(high)))) if l != r => {
                    return Err(Failure {
                        parse_error: ParseError::new("invalid group range")
                            .with_span(
                                "range bounds are in different groups", 
                                succ.lexer.span(),
                                succ.lexer.column_metrics()),
                        lexer: succ.lexer,
                        source: None,
                    })
                },
                
                ((l, Some(low)), Some((r, Some(high)))) => Ok(succ)
                    .map_value(|_| GroupRange { group: l, low, high }),

                ((_, None),      Some((_, _)))    |
                ((_, _),         Some((_, None))) => return Err(Failure {
                    parse_error: ParseError::new("invalid range")
                        .with_span(
                            "range bounds have incompatable types", 
                            succ.lexer.span(),
                            succ.lexer.column_metrics()),
                    lexer: succ.lexer,
                    source: None,
                }),
                
                ((l, Some(idx)), None) => Ok(succ)
                    .map_value(|_| Group { group: l, idx }),

                ((l, None),      None) => Ok(succ)
                    .map_value(|_| Name(l)),
            }
        },

        _ => fail
            (lexer)
            .map_value(|_| unreachable!())
    }
}

pub fn position_selector<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, PositionSelector>
    where Cm: ColumnMetrics,
{
    exact(
        right(one(AtmaToken::Colon),
            both(
                uint_16_or_all,
                both(
                    right(one(AtmaToken::Decimal), uint_16_or_all),
                    right(one(AtmaToken::Decimal), uint_16_or_all)))))
        (lexer)
        .map_value(|(page, (line, column))| PositionSelector {
             page,
             line,
             column,
        })
}

fn uint_16_or_all<'text, Cm>(mut lexer: Lexer<'text, AtmaScanner, Cm>)
    -> ParseResult<'text, AtmaScanner, Cm, Option<u16>>
    where Cm: ColumnMetrics,
{
    if let Ok(succ) = one(AtmaToken::Mult)(lexer.clone()) {
        return Ok(succ).map_value(|_| None);
    }

    uint::<_, u16>
        (lexer)
        .map_value(Some)
}

fn range<'text, Cm, F, V: std::fmt::Debug>(mut parser: F)
    -> impl FnMut(Lexer<'text, AtmaScanner, Cm>)
        -> ParseResult<'text, AtmaScanner, Cm, (V, Option<V>)>
    where
        Cm: ColumnMetrics,
        F: FnMut(Lexer<'text, AtmaScanner, Cm>)
            -> ParseResult<'text, AtmaScanner, Cm, V>,
{
    move |lexer| {
        let (l, succ) = (&mut parser)
            (lexer)?
            .take_value();

        atomic(
            right(
                one(AtmaToken::Minus),
                &mut parser))
            (succ.lexer)
            .map_value(|r| (l, r))
    }
}
