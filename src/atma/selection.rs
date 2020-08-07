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
use crate::atma::AtmaScanner;
use crate::atma::AtmaToken;
use crate::atma::CellRef;
use crate::atma::CellSelection;
use crate::atma::CellSelector;
use crate::atma::Position;
use crate::atma::PositionSelector;
use crate::atma::string;
use crate::atma::uint;
use crate::combinator::any;
use crate::combinator::atomic;
use crate::combinator::both;
use crate::combinator::bracket;
use crate::combinator::bracket_dynamic;
use crate::combinator::exact;
use crate::combinator::intersperse_collect;
use crate::combinator::left;
use crate::combinator::maybe;
use crate::combinator::one;
use crate::combinator::right;
use crate::combinator::section;
use crate::combinator::seq;
use crate::combinator::text;
use crate::lexer::Lexer;
use crate::result::Failure;
use crate::result::ParseError;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::span::NewLine;

// Standard library imports.
use std::borrow::Cow;


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

pub fn cell_ref<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, CellRef<'text>>
    where Nl: NewLine,
{
    match position
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(CellRef::Position),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    // Index must come after position.
    match index
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(CellRef::Index),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    group_or_name
        (lexer)
        .map_value(|(name, idx)| match idx {
            Some(idx) => CellRef::Group { group: name, idx },
            None      => CellRef::Name(name),
        })
}

pub fn index<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, u32>
    where Nl: NewLine,
{
    exact(
        right(one(AtmaToken::Colon),
            uint::<_, u32>))
        (lexer)
}


pub fn position<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, Position>
    where Nl: NewLine,
{
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

pub fn group_or_name<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, (Cow<'text, str>, Option<u32>)>
    where Nl: NewLine,
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

pub fn cell_selection<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, CellSelection<'text>>
    where Nl: NewLine,
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

pub fn cell_selector<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, CellSelector<'text>>
    where Nl: NewLine,
{
    use CellSelector::*;

    match exact(
        left(
            string,
            seq(&[AtmaToken::Colon, AtmaToken::Mult])))
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(|name| GroupAll(name)),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    match range(position)
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ) => {
            let (val, succ) = succ.take_value();
            match val {
                (low, Some(high)) if low > high => unimplemented!(),

                (low, Some(high)) => return Ok(succ)
                    .map_value(|_| PositionRange { low, high }),

                (pos, None)       => return Ok(succ)
                    .map_value(|_| PositionSelector(pos.into())),
            }
        },
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    // PositionSelector must come after PositionRange.
    match position_selector
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(PositionSelector),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    // Index must come after PositionRange and PositionSelector.
    match range(index)
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ) => {
            let (val, succ) = succ.take_value();
            match val {
                (low, Some(high)) if low > high => unimplemented!(),

                (low, Some(high)) => return Ok(succ)
                    .map_value(|_| IndexRange { low, high }),

                (idx, None)       => return Ok(succ)
                    .map_value(|_| Index(idx)),
            }
        },
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    // All must come after PositionSelector. 
    match exact(
            seq(&[AtmaToken::Colon, AtmaToken::Mult]))
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(|_| All),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }

    // Group and Name must come after GroupAll.
    let (val, succ) = range(group_or_name)
        (lexer.clone())?
        .take_value();
    match val {
        ((l, Some(low)), Some((r, Some(high)))) if low > high => unimplemented!(),
        ((l, Some(low)), Some((r, Some(high)))) if l != r => unimplemented!(),
        
        ((l, Some(low)), Some((r, Some(high)))) => Ok(succ)
            .map_value(|_| GroupRange { group: l, low, high }),

        ((_, None),      Some((_, _)))    => unimplemented!(),
        ((_, _),         Some((_, None))) => unimplemented!(),
        
        ((l, Some(idx)), None) => Ok(succ)
            .map_value(|_| Group { group: l, idx }),

        ((l, None),      None) => Ok(succ)
            .map_value(|_| Name(l)),
    }
}

pub fn position_selector<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, PositionSelector>
    where Nl: NewLine,
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

fn uint_16_or_all<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, Option<u16>>
    where Nl: NewLine,
{
    if let Ok(succ) = one(AtmaToken::Mult)(lexer.clone()) {
        return Ok(succ).map_value(|_| None);
    }

    uint::<_, u16>
        (lexer)
        .map_value(Some)
}

fn range<'text, Nl, F, V>(mut parser: F)
    -> impl FnMut(Lexer<'text, AtmaScanner, Nl>)
        -> ParseResult<'text, AtmaScanner, Nl, (V, Option<V>)>
    where
        Nl: NewLine,
        F: FnMut(Lexer<'text, AtmaScanner, Nl>)
            -> ParseResult<'text, AtmaScanner, Nl, V>,
{
    move |lexer| {
        let (l, succ) = (&mut parser)
            (lexer)?
            .take_value();

        maybe(
            right(
                one(AtmaToken::Minus),
                &mut parser))
            (succ.lexer)
            .map_value(|r| (l, r))
    }
}
