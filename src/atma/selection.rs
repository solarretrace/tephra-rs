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
use crate::atma::AtmaToken;
use crate::atma::AtmaScanner;
use crate::atma::string;
use crate::atma::uint;
use crate::atma::Position;
use crate::atma::PositionSelector;
use crate::atma::CellRef;
use crate::lexer::Lexer;
use crate::span::NewLine;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::result::ParseError;
use crate::result::Failure;
use crate::combinator::one;
use crate::combinator::any;
use crate::combinator::bracket_dynamic;
use crate::combinator::bracket;
use crate::combinator::text;
use crate::combinator::both;
use crate::combinator::exact;
use crate::combinator::right;

////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////

pub fn cell_ref<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, CellRef<'text>>
    where Nl: NewLine,
{
    if let Ok(succ) = index(lexer.clone()) {
        return Ok(succ).map_value(CellRef::Index);
    }
    if let Ok(succ) = position(lexer.clone()) {
        return Ok(succ).map_value(CellRef::Position);
    }

    if let Ok(succ) = string(lexer.clone()) {
        return Ok(succ).map_value(CellRef::Name);
    }

    exact(
        both(
            string,
            right(one(AtmaToken::Colon),
                uint::<_, u32>)))
        (lexer)
        .map_value(|(group, idx)| CellRef::Group { group, idx })
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


////////////////////////////////////////////////////////////////////////////////
// CellSelector
////////////////////////////////////////////////////////////////////////////////



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
