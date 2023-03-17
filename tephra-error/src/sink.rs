

use crate::ParseError;
use crate::Context;

use std::rc::Rc;
use parking_lot::RwLock;



////////////////////////////////////////////////////////////////////////////////
// ErrorSink
////////////////////////////////////////////////////////////////////////////////
/// A target to send parse errors to be processed by the application.
#[derive(Debug)]
#[repr(transparent)]
pub struct ErrorSink<'text>(Rc<ErrorSinkInner<'text>>);

impl<'text> ErrorSink<'text> {
    /// Constructs a new `ErrorSink` that processes errors via the given
    /// function.
    ///
    /// The target function is called for every error sent using the
    /// `send` or `send_direct` methods.
    pub fn new<T>(target: T) -> Self
        where T: Fn(ParseError<'text>) + 'static
    {
        ErrorSink(Rc::new(ErrorSinkInner {
            contexts: RwLock::new(Vec::new()),
            target: Box::new(target),
        }))
    }

    /// Sends a `ParseError` to the sink target, wrapping it in any available
    /// `Context`s.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn send<'a>(&'a self, parse_error: ParseError<'text>)
        -> Result<(), Box<dyn std::error::Error + 'a>>
    {
        let inner = self.0.as_ref();

        let mut e = parse_error;
        let contexts = inner.contexts.read();
        for context in contexts.iter().rev() {
            e = context.apply(e);
        }

        (inner.target)(e);
        Ok(())
    }

    /// Sends a `ParseError` to the sink target without wrapping it in any
    /// `Context`s.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn send_direct<'a>(&'a self, parse_error: ParseError<'text>) {
        let inner = self.0.as_ref();
        let e = parse_error;

        (inner.target)(e);
    }

    /// Pushes a new `Context` onto the context stack, allowing any further
    /// `ParseError`s to be processed by them. 
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn push_context<'a>(&'a mut self, context: Context<'text>) {
        let mut contexts = self.0.as_ref().contexts.write();
        contexts.push(context);
    }

    /// Pops the top `Context` from the context stack.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn pop_context<'a>(&'a mut self) -> Option<Context<'text>> {
        let mut contexts = self.0.as_ref().contexts.write();
        contexts.pop()
    }


    /// Replaces the top `Context` from the context stack and returns the old
    /// value.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn replace_context<'a>(&'a mut self, context: Context<'text>)
        -> Option<Context<'text>>
    {
        let mut contexts = self.0.as_ref().contexts.write();
        let old = contexts.pop();
        contexts.push(context);
        old
    }

    /// Replaces the the context stack and returns the old value.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn replace_contexts<'a>(&'a mut self, new_contexts: Vec<Context<'text>>)
        -> Vec<Context<'text>>
    {
        let mut contexts = self.0.as_ref().contexts.write();
        std::mem::replace(&mut contexts, new_contexts)
    }


    pub fn apply_current_context<'a>(
        &'a self,
        parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        match self.0.as_ref().contexts.read().last() {
            Some(context) => context.apply(parse_error),
            None          => parse_error,
        }
    }
}

impl<'text> Clone for ErrorSink<'text> {
    fn clone(&self) -> Self {
        ErrorSink(Rc::clone(&self.0))
    }
}


////////////////////////////////////////////////////////////////////////////////
// ErrorSinkInner
////////////////////////////////////////////////////////////////////////////////
struct ErrorSinkInner<'text> {
    contexts: RwLock<Vec<Context<'text>>>,
    target: Box<dyn Fn(ParseError<'text>)>,
}


impl<'text> std::fmt::Debug for ErrorSinkInner<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorSinkInner")
            .field("contexts", &self.contexts)
            .field("target", &"...")
            .finish()
    }
}




