////////////////////////////////////////////////////////////////////////////////
// Tephra parser library
////////////////////////////////////////////////////////////////////////////////
// Copyright 2022 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse results.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
// mod display;
mod source_display;
mod error;
mod ext;
mod failure;
mod success;

// Exports.
pub use self::source_display::*;
pub use self::error::*;
pub use self::ext::*;
pub use self::failure::*;
pub use self::success::*;




/*
Parse failures can occur with varying levels of contextual information. 

### Lexer Errors

The least informative parse errors are 'lexer errors', which includes the four
 errors:

1. `unrecognized_token`: A token failed to lex.
2. `unexpected_token`: The wrong token was lexed.
3. `unexpected_end_of_text`: The end of text was encountered unexpectedly.
4. `expected_end_of_text`: The end of text was not encountered where expected.

Lexer errors are emitted by default, and require no special action to handle.
All other errors require user intervention.

### Contextual Errors
A contextual error is any error that occurs during a parse in which we want to
expose some part of the parsing as an external grammatical element. That is, we
have provided context to another error by including names for the parts of the
grammar we are currently parsing on. 

Contexts have precedence:
1. Bounded Context
2. Delimited Context
3. Unbounded (Recursive) Context
4. No context (lexer errors)

If the current error has a higher precedence than the context, the context is
replaces the error output. Otherwise, it suppliments to it.

### Partial Success Errors
A partial success error is produced when we have successfully parsed some prefix
of a unique production, and can thus establish that the error encountered is
exactly where a correction can occur. They are also bounded contextual errors,
and provide the most useful context to an error.

In tephra, these are modeled by the `atomic` combinator.

### Branching Errors
Branching errors occur whenever there are multiple possible parses, and we
intend to find only one that succeeds. All other failures are branching errors.

It can be difficult to report on branching errors because we will (by default)
throw them away and try something else. But what do we return if all branches
fail? In theory, we should return the error for the branch that makes the most
progress, but that is difficult to measure.

A branching error should always devolve into a contextual error: we should
establish context for what we're parsing before parsing multiple possibilities,
so that we can report what we expected without showing errors for any specific
branch. The key exception is partial success: if a partial success error is
encountered on one of the branches, we know that no other branches could
succeed, so we can fail early and report the partial success context. Thus it is
very important to try to build branching grammars around distinct prefixes.

### Validation Errors

A validation error is any error manually introduced by a parser. This is usually
done whenever a parse fails due to a fallible conversion or some other
grammar-specific condition. These errors behave much like lexer errors, in that
they provide very little context, but a user might be able to tell what is wrong
depending on the exact error produced.

*/
