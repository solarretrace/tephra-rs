////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error recovery.
////////////////////////////////////////////////////////////////////////////////


// Standard library imports.
use std::fmt::Debug;
use std::fmt::Display;






////////////////////////////////////////////////////////////////////////////////
// Recover
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Recover<T>
    where T: Display + Debug + Clone + PartialEq + Send + Sync + 'static
{
    Wait,
    Before {
        token: T,
    },
    After {
        token: T,
    },
    BeforeLimit {
        token: T,
        limit: u32,
    },
    AfterLimit {
        token: T,
        limit: u32,
    },
}

impl<T> Recover<T>
    where T: Display + Debug + Clone + PartialEq + Send + Sync + 'static
{
    pub fn before(token: T) -> Self {
        Recover::Before { token }
    }

    pub fn after(token: T) -> Self {
        Recover::After { token }
    }

    pub fn is_recovering(&self) -> bool {
        *self == Recover::Wait
    }

    pub fn limit(&self) -> Option<&u32> {
        match self {
            Recover::BeforeLimit { limit, .. } |
            Recover::AfterLimit { limit, .. }  => Some(limit),
            _ => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// RecoverError
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy)]
pub enum RecoverError {
    LimitExceeded,
    EndOfText,
}

impl RecoverError {
    pub fn description(&self) -> &str {
        match self {
            RecoverError::LimitExceeded
                => "recover failure: too many failed attempts",
            RecoverError::EndOfText
                => "recover failure: end of text reached",
        }
    }
}

impl<'text> std::fmt::Display for RecoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl<'text> std::error::Error for RecoverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
