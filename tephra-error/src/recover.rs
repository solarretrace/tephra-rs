////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error recovery.
////////////////////////////////////////////////////////////////////////////////

// External library imports.
use simple_predicates::Expr;
use simple_predicates::Eval;
use simple_predicates::DnfVec;
// Standard library imports.
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    Before,
    After,
}


#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Token<T>(T);

impl<T> Eval for Token<T> where T: Clone + PartialEq {
    type Context = T;

    fn eval(&self, data: &Self::Context) -> bool {
        &self.0 == data
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Recover<T> where T: Clone + PartialEq {
    before: DnfVec<Token<T>>,
    after: DnfVec<Token<T>>,
    limit: Option<u32>,
}

impl<T> Recover<T>
    where T: Display + Debug + Clone + PartialEq + Send + Sync + 'static
{
    pub fn empty() -> Self {
        Recover {
            before: DnfVec::from(None),
            after: DnfVec::from(None),
            limit: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }

    pub fn check(&self, token: &T) -> Option<PositionType> {
        if self.is_empty() { return None; }

        if self.before.eval(token) {
            Some(PositionType::Before)
        } else if self.after.eval(token) {
            Some(PositionType::After)
        } else {
            None
        }
    }

    pub fn before(token: T) -> Self {
        Recover {
            before: DnfVec::from(std::iter::once(Expr::Var(Token(token)))),
            after: DnfVec::from(None),
            limit: None,
        }
    }

    pub fn after(token: T) -> Self {
        Recover {
            before: DnfVec::from(None),
            after: DnfVec::from(std::iter::once(Expr::Var(Token(token)))),
            limit: None,
        }
    }

    pub fn before_any<I>(tokens: I) -> Self
        where I: IntoIterator<Item=T>
    {
        let tokens = tokens.into_iter().map(Token).map(Expr::Var);
        Recover {
            before: DnfVec::from(tokens),
            after: DnfVec::from(None),
            limit: None,
        }
    }

    pub fn after_any<I>(tokens: I) -> Self
        where I: IntoIterator<Item=T>
    {
        let tokens = tokens.into_iter().map(Token).map(Expr::Var);
        Recover {
            before: DnfVec::from(None),
            after: DnfVec::from(tokens),
            limit: None,
        }
    }

    pub fn before_expr(expr: Expr<T>) -> Self {
        Recover {
            before: DnfVec::from(expr.map(Token)),
            after: DnfVec::from(None),
            limit: None,
        }
    }

    pub fn after_expr(expr: Expr<T>) -> Self {
        Recover {
            before: DnfVec::from(None),
            after: DnfVec::from(expr.map(Token)),
            limit: None,
        }
    }

    pub fn limit(&self) -> Option<u32> {
        self.limit
    }

    pub fn limit_mut(&mut self) -> &mut Option<u32> {
        &mut self.limit
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
