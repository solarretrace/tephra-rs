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



// pub fn index<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
//     -> ParseResult<'text, AtmaScanner, Nl, CellRef>
//     where Nl: NewLine,
// {
//     exact(
//         right(one(AtmaToken::Colon),
//             uint::<_, u32>))
//         (lexer)
//         .map_value(CellRef::Index)
// }


// pub fn position<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
//     -> ParseResult<'text, AtmaScanner, Nl, CellRef>
//     where Nl: NewLine,
// {
//     exact(
//         right(one(AtmaToken::Colon),
//             both(
//                 uint::<_, u16>,
//                 both(
//                     right(one(AtmaToken::Decimal), uint::<_, u16>),
//                     right(one(AtmaToken::Decimal), uint::<_, u16>)))))
//         (lexer)
//         .map_value(|(p, (l, c))| CellRef::Position(p, l, c))
// }


// pub fn group<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
//     -> ParseResult<'text, AtmaScanner, Nl, CellRef>
//     where Nl: NewLine,
// {
//     exact(
//         both(
//             string,
//             right(one(AtmaToken::Colon),
//                 uint::<_, u32>)))
//         (lexer)
//         .map_value(|(name, idx)| CellRef::Group(name.to_string(), idx))
// }

// pub fn name<'text, Nl>(mut lexer: Lexer<'text, AtmaScanner, Nl>)
//     -> ParseResult<'text, AtmaScanner, Nl, CellRef>
//     where Nl: NewLine,
// {
//     string
//         (lexer)
//         .map_value(|name| CellRef::Name(name.to_string()))
// }
