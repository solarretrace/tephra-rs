////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error recovery.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::error::RecoverError;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;



// Ok(true) => recover is finished, resume parsing
// Ok(false) => recover is not finished, keep advancing
// Err(&str) => recover failed, 

pub type Recover<T>
    = Rc<RwLock<dyn FnMut(T) -> Result<bool, RecoverError>>>;

pub fn recover_after<T>(token: T) -> Recover<T> 
    where T: PartialEq + Send + Sync + 'static
{
    let mut found = false;

    Rc::new(RwLock::new(move |next_token| {
        if found {
            Ok(true)
        } else {
            found = next_token == token;
            Ok(false)
        }
    }))
}

pub fn recover_before<T>(token: T) -> Recover<T> 
    where T: PartialEq + Send + Sync + 'static,
{
    Rc::new(RwLock::new(move |next_token| {
        Ok(next_token == token)
    }))
}


pub fn recover_after_any<T, I>(tokens: I) -> Recover<T> 
    where
        T: PartialEq + Send + Sync + 'static,
        I: IntoIterator<Item=T>,
{
    let mut found = false;
    let tokens: Vec<_> = tokens.into_iter().collect();

    Rc::new(RwLock::new(move |next_token| {
        if found {
            Ok(true)
        } else {
            found = tokens.contains(&next_token);
            Ok(false)
        }
    }))
}

pub fn recover_before_any<T, I>(tokens: I) -> Recover<T> 
    where
        T: PartialEq + Send + Sync + 'static,
        I: IntoIterator<Item=T>,
{
    let tokens: Vec<_> = tokens.into_iter().collect();

    Rc::new(RwLock::new(move |next_token| {
        Ok(tokens.contains(&next_token))
    }))
}


