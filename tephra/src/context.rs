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
pub type ErrorSink<'text> = Box<dyn Fn(ParseError<'text>)>;

////////////////////////////////////////////////////////////////////////////////
// ErrorTransform
////////////////////////////////////////////////////////////////////////////////
pub type ErrorTransform<'text> = Rc<dyn Fn(ParseError<'text>)
    -> ParseError<'text>>;




////////////////////////////////////////////////////////////////////////////////
// SharedContext
////////////////////////////////////////////////////////////////////////////////
struct SharedContext<'text> {
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
pub struct LocalContext<'text> {
    error_transform: Option<ErrorTransform<'text>>,
    parent: Option<Rc<RwLock<LocalContext<'text>>>>,
}

impl<'text> LocalContext<'text> {
    fn apply_error_transform(&self, parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        match self.error_transform.as_ref() {
            Some(transform) => (transform)(parse_error),
            None            => parse_error,
        }
    }

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
#[derive(Debug, Clone)]
pub struct Context<'text> {
    shared: Rc<RwLock<SharedContext<'text>>>,
    local: Rc<RwLock<LocalContext<'text>>>,
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
        }
    }

    /// Constructs a new `Context` by wrapping a new `ErrorTransform` around the
    /// given `Context`.
    pub fn push(self, error_transform: ErrorTransform<'text>) -> Self {
        Context {
            shared: self.shared.clone(),
            local: Rc::new(RwLock::new(LocalContext {
                parent: Some(self.local.clone()),
                error_transform: Some(error_transform),
            })),
        }
    }

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

    fn apply_error_transform_recursive(&self, parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        self.local
            .read()
            .expect("read local context")
            .apply_error_transform_recursive(parse_error)
    }

    /// Sends a `ParseError` to the `ErrorSink`, applying `ErrorTransform`s.
    pub fn send_error<'a>(&'a self, parse_error: ParseError<'text>)
    {
        if let Some(sink) =  self.shared
            .read()
            .expect("lock shared context")
            .error_sink
            .as_ref()
        {
            (sink)(self.apply_error_transform_recursive(parse_error));
        }
    }

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

