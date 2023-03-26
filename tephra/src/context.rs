////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse error.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(missing_docs)]


// External library imports.
use tephra_error::ParseError;

// Standard library imports.
use std::rc::Rc;
use std::sync::RwLock;


fn option_fmt<T>(opt: &Option<T>) -> &'static str {
    if opt.is_some() {"Some(...)"} else {"None"}
}

////////////////////////////////////////////////////////////////////////////////
// ErrorSink
////////////////////////////////////////////////////////////////////////////////
/// A function which can receive recoverable `ParseError`s.
pub type ErrorSink<'text> = Box<dyn Fn(ParseError<'text>) + 'text>;

////////////////////////////////////////////////////////////////////////////////
// ErrorTransform
////////////////////////////////////////////////////////////////////////////////
/// A function to construct or modify `ParseError`s in a given `Context`.
pub type ErrorTransform<'text>
    = Rc<dyn Fn(ParseError<'text>) -> ParseError<'text> + 'text>;



////////////////////////////////////////////////////////////////////////////////
// SharedContext
////////////////////////////////////////////////////////////////////////////////
/// Shared parse context.
struct SharedContext<'text> {
    /// The `ErrorSink` function.
    error_sink: Option<ErrorSink<'text>>,
}

impl<'text> std::fmt::Debug for SharedContext<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedContext")
            .field("error_sink", &option_fmt(&self.error_sink))
            .finish()
    }
}

////////////////////////////////////////////////////////////////////////////////
// LocalContext
////////////////////////////////////////////////////////////////////////////////
/// Local parse context. Forms a linked list of parse contexts back to the start
/// of the parse.
pub struct LocalContext<'text> {
    /// The lowest `ErrorTransform` function in this context.
    error_transform: Option<ErrorTransform<'text>>,
    /// The `LocalContext` of the next highest parse.
    parent: Option<Rc<RwLock<LocalContext<'text>>>>,
}

impl<'text> LocalContext<'text> {
    /// Applies the lowest `ErrorTransform` function to the given `ParseError`.
    fn apply_error_transform(&self, parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        match self.error_transform.as_ref() {
            Some(transform) => (transform)(parse_error),
            None            => parse_error,
        }
    }

    /// Applies the lowest `ErrorTransform` function to the given `ParseError`,
    /// then visits each parent context and applies each transfom function in
    /// sequence.
    fn apply_error_transform_recursive(&self, parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        let e = self.apply_error_transform(parse_error);

        match self.parent.as_ref() {
            Some(parent) => parent
                .read()
                .expect("read parent")
                .apply_error_transform_recursive(e),
            None         => e,

        }
    }
}


impl<'text> std::fmt::Debug for LocalContext<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedContext")
            .field("error_transform", &option_fmt(&self.error_transform))
            .field("parent", &self.parent)
            .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////
// Context
////////////////////////////////////////////////////////////////////////////////
/// A parse context.
#[derive(Debug, Clone)]
pub struct Context<'text> {
    /// The `SharedContext`.
    shared: Rc<RwLock<SharedContext<'text>>>,
    /// The `LocalContext`.
    local: Rc<RwLock<LocalContext<'text>>>,
    /// Indicates that the context is locked and no new contexts may be added.
    locked: bool,
}

impl<'text> Context<'text> {
    /// Constructs a new `Context`.
    pub fn empty() -> Self {
        Context {
            shared: Rc::new(RwLock::new(SharedContext {
                error_sink: None,
            })),
            local: Rc::new(RwLock::new(LocalContext {
                error_transform: None,
                parent: None,
            })),
            locked: false,
        }
    }

    /// Constructs a new `Context` with the given `ErrorSink`.
    pub fn new(error_sink: Option<ErrorSink<'text>>) -> Self {
        Context {
            shared: Rc::new(RwLock::new(SharedContext {
                error_sink,
            })),
            local: Rc::new(RwLock::new(LocalContext {
                error_transform: None,
                parent: None,
            })),
            locked: false,
        }
    }

    /// Sets the lock value of the `Context`. The value indicates that the
    /// whether new contexts may be pushed.
    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }


    /// Constructs a new `Context` by wrapping a new `ErrorTransform` around the
    /// given `Context`.
    pub fn push(self, error_transform: ErrorTransform<'text>) -> Self {
        if !self.locked {
            Context {
                shared: self.shared.clone(),
                local: Rc::new(RwLock::new(LocalContext {
                    parent: Some(self.local.clone()),
                    error_transform: Some(error_transform),
                })),
                locked: false,
            }
        } else {
            self
        }
    }

    /// Removes the `ErrorSink` from the `Context` if present.
    pub fn take_error_sink(&mut self) -> Option<ErrorSink<'text>> {
        let mut shared = self.shared.write().expect("lock shared context");
        shared.error_sink.take()
    }

    pub fn replace_error_sink(&mut self, error_sink: ErrorSink<'text>)
        -> Option<ErrorSink<'text>>
    {
        let mut shared = self.shared.write().expect("lock shared context");
        shared.error_sink.replace(error_sink)
    }

    fn apply_error_transform(&self, parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        self.local
            .read()
            .expect("read local context")
            .apply_error_transform(parse_error)
    }

    pub fn apply_error_transform_recursive(
        &self,
        parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        self.local
            .read()
            .expect("read local context")
            .apply_error_transform_recursive(parse_error)
    }

    /// Sends a `ParseError` to the `ErrorSink`, applying `ErrorTransform`s.
    ///
    /// Returns the given error if no `ErrorSink` is configured.
    pub fn send_error<'a>(&'a self, parse_error: ParseError<'text>)
        -> Result<(), ParseError<'text>>
    {
        match self.shared
            .read()
            .expect("lock shared context")
            .error_sink
            .as_ref()
        {
            Some(sink) => {
                (sink)(self.apply_error_transform_recursive(parse_error));
                Ok(())
            },
            None => Err(parse_error),
        }
    }

    /// Removes the `LocalContext` from the `Context` if present.
    pub fn take_local_context(&mut self) -> LocalContext<'text> {
        std::mem::replace(&mut *self.local
                .write()
                .expect("write local context"),
            LocalContext {
                error_transform: None,
                parent: None,
            })
    }

    pub fn replace_local_context(&mut self, local: LocalContext<'text>)
        -> LocalContext<'text>
    {
        std::mem::replace(&mut *self.local
                .write()
                .expect("write local context"),
            local)
    }
}
