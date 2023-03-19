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
// Context
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct Context<'text> {
    shared: Rc<SharedContext<'text>>,
    error_transform: Option<ErrorTransform<'text>>,
    parent: Option<Rc<Context<'text>>>,
}


impl<'text> Context<'text> {
    pub fn empty() -> Self {
        Context {
            shared: Rc::new(SharedContext {
                error_sink: None,
            }),
            error_transform: None,
            parent: None,
        }
    }

    pub fn with_error_sink(mut self, error_sink: ErrorSink<'text>)
        -> Self
    {
        if let Some(shared) = Rc::get_mut(&mut self.shared) {
            shared.error_sink  = Some(error_sink);
        }
        self
    }

    pub fn error_sink(&self) -> &Option<ErrorSink<'text>> {
        &self.shared.error_sink
    }

    pub fn take_error_sink(&mut self) -> Option<ErrorSink<'text>> {
        Rc::get_mut(&mut self.shared)
            .and_then(|shared| shared.error_sink.take())
    }

    pub fn replace_error_sink(&mut self, error_sink: ErrorSink<'text>)
        -> Option<ErrorSink<'text>>
    {
        Rc::get_mut(&mut self.shared)
            .and_then(|shared| shared.error_sink.replace(error_sink))
    }

    pub fn error_transform(&self) -> &Option<ErrorTransform<'text>> {
        &self.error_transform
    }

    pub fn error_transform_mut(&mut self)
        -> &mut Option<ErrorTransform<'text>>
    {
        &mut self.error_transform
    }

    pub fn parent(&self) -> &Option<Rc<Context<'text>>> {
        &self.parent
    }

    pub fn parent_mut(&mut self)
        -> &mut Option<Rc<Context<'text>>>
    {
        &mut self.parent
    }

    pub fn push_context(self, error_transform: ErrorTransform<'text>) -> Self {
        Context {
            shared: self.shared.clone(),
            error_transform: Some(error_transform),
            parent: Some(Rc::new(self)),
        }
    }

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
            Some(parent) => parent.apply_error_transform_recursive(e),
            None         => e,

        }
    }

    /// Sends a `ParseError` to the `ErrorSink`, applying `ErrorTransform`s.
    pub fn send_error<'a>(&'a self, parse_error: ParseError<'text>)
    {
        if let Some(sink) = self.error_sink().as_ref() {
            (sink)(self.apply_error_transform_recursive(parse_error));
        }
    }
}


fn option_fmt<T>(opt: &Option<T>) -> &'static str {
    if opt.is_some() {"Some(...)"} else {"None"}
}

impl<'text> std::fmt::Debug for Context<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("shared", &self.shared)
            .field("error_transform", &option_fmt(&self.error_transform))
            .field("parent", &self.parent)
            .finish()
    }
}



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
