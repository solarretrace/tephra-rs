////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Atma intermediate function call.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::atma::AtmaScanner;
use crate::atma::AtmaToken;
use crate::atma::FnArg;
use crate::atma::FnCall;
use crate::atma::uint;
use crate::atma::float;
use crate::combinator::both;
use crate::combinator::intersperse_collect;
use crate::combinator::one;
use crate::combinator::section;
use crate::combinator::with_span;
use crate::combinator::text;
use crate::combinator::bracket;
use crate::lexer::Lexer;
use crate::result::ParseResult;
use crate::result::ParseResultExt as _;
use crate::position::NewLine;


////////////////////////////////////////////////////////////////////////////////
// FnCall
////////////////////////////////////////////////////////////////////////////////

pub fn fn_call<'text, Nl>(lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, FnCall<'text, Nl>>
    where Nl: NewLine,
{
    both(
        text(one(AtmaToken::Ident)),
        bracket(
            one(AtmaToken::OpenParen),
            intersperse_collect(0, None,
                section(with_span(fn_arg)),
                one(AtmaToken::Comma)),
            one(AtmaToken::CloseParen)))
        (lexer)
        .map_value(|(name, args)| FnCall { name, args })
}

pub fn fn_arg<'text, Nl>(lexer: Lexer<'text, AtmaScanner, Nl>)
    -> ParseResult<'text, AtmaScanner, Nl, FnArg>
    where Nl: NewLine,
{
    match float::<_, f32>
        (lexer.clone())
        .filter_lexer_error()
    {
        Ok(succ)        => return Ok(succ).map_value(FnArg::F32),
        Err(Some(fail)) => return Err(fail),
        Err(None)       => (),
    }


    uint::<_, u32>
        (lexer)
        .map_value(FnArg::U32)
}
