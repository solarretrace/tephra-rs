

use crate::ParseError;

use std::rc::Rc;
use std::sync::RwLock;



////////////////////////////////////////////////////////////////////////////////
// ErrorSink
////////////////////////////////////////////////////////////////////////////////
/// A target to send parse errors to be processed by the application.
#[derive(Debug)]
#[repr(transparent)]
pub struct ErrorSink(Rc<ErrorSinkInner>);

impl ErrorSink {
    /// Constructs a new `ErrorSink` that processes errors via the given
    /// function.
    ///
    /// The target function is called for every error sent using the
    /// `send` or `send_direct` methods.
    pub fn new<T>(target: T) -> Self
        where T: for<'text> Fn(ParseError<'text>) + 'static
    {
        ErrorSink(Rc::new(ErrorSinkInner {
            contexts: RwLock::new(Vec::new()),
            target: Box::new(target),
        }))
    }

    /// Sends a `ParseError` to the sink target, wrapping it in any available
    /// `ErrorContext`s.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn send<'a, 'text>(&'a self, parse_error: ParseError<'text>)
        -> Result<(), Box<dyn std::error::Error + 'a>>
    {
        let inner = self.0.as_ref();

        let mut e = parse_error;
        let contexts = inner.contexts.read()?;
        for context in contexts.iter().rev() {
            e = context.apply(e);
        }

        (inner.target)(e);
        Ok(())
    }

    /// Sends a `ParseError` to the sink target without wrapping it in any
    /// `ErrorContext`s.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn send_direct<'a, 'text>(&'a self,
        parse_error: ParseError<'text>)
        -> Result<(), Box<dyn std::error::Error + 'a>>
    {
        let inner = self.0.as_ref();
        let e = parse_error;

        (inner.target)(e);
        Ok(())
    }

    /// Pushes a new `ErrorContext` onto the context stack, allowing any further
    /// `ParseError`s to be processed by them. 
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn push_context<'a>(&'a mut self, error_context: ErrorContext) 
        -> Result<(), Box<dyn std::error::Error + 'a>>
    {
        let mut contexts = self.0.as_ref().contexts.write()?;
        contexts.push(error_context);
        Ok(())
    }

    /// Pops the top `ErrorContext` from the context stack.
    ///
    /// Returns an error if the internal sink [RwLock] has been poisoned.
    ///
    /// [RwLock]: https://doc.rust-lang.org/stable/std/sync/struct.RwLock.html
    pub fn pop_context<'a>(&'a mut self) 
        -> Result<Option<ErrorContext>, Box<dyn std::error::Error + 'a>>
    {
        let mut contexts = self.0.as_ref().contexts.write()?;
        Ok(contexts.pop())
    }
}

impl Clone for ErrorSink {
    fn clone(&self) -> Self {
        ErrorSink(Rc::clone(&self.0))
    }
}


////////////////////////////////////////////////////////////////////////////////
// ErrorSinkInner
////////////////////////////////////////////////////////////////////////////////
struct ErrorSinkInner {
    contexts: RwLock<Vec<ErrorContext>>,
    target: Box<dyn for<'text> Fn(ParseError<'text>)>,
}


impl std::fmt::Debug for ErrorSinkInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorSinkInner")
            .field("contexts", &self.contexts)
            .field("target", &"...")
            .finish()
    }
}



////////////////////////////////////////////////////////////////////////////////
// ErrorContext
////////////////////////////////////////////////////////////////////////////////
pub struct ErrorContext {
    name: &'static str,
    apply_fn: Rc<dyn for<'text> Fn(ParseError<'text>) -> ParseError<'text>>,
}

impl ErrorContext {
    pub fn apply<'text>(&self, parse_error: ParseError<'text>)
        -> ParseError<'text>
    {
        (self.apply_fn)(parse_error)
    }
}

impl std::fmt::Debug for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorContext")
            .field("name", &self.name)
            .field("apply_fn", &"...")
            .finish()
    }
}

impl Clone for ErrorContext {
    fn clone(&self) -> Self {
        ErrorContext {
            name: self.name,
            apply_fn: Rc::clone(&self.apply_fn),
        }
    }
}

impl From<&'static str> for ErrorContext {
    fn from(name: &'static str) -> Self {
        ErrorContext {
            name,
            apply_fn: Rc::new(move |e| e),
        }
    }
}
