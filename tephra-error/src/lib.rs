////////////////////////////////////////////////////////////////////////////////
// Tephra parser errors library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Tephra parse errors.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]




// Internal modules.
#[cfg(test)]
mod test;
mod code;
mod error;
mod sink;
mod highlight;
mod message;

// Exports.
pub use error::*;
pub use sink::*;
pub use code::*;
pub use message::*;
pub use highlight::*;
