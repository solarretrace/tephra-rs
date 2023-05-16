
# Terminology

## Parser
## Combinator / Primitive
## Lexer
## Scanner
## Token
## NewLine
## Span
## Position / Pos
## Source
## ParseResult / Success / Failure
## OwnedFailure / OwnedSpan
## SpanDisplay / Highlight


# Parser design principles

    Designed to be user-friendly while reasonably efficient (wrt backtracking) and flexable.

    Minimalistic combinator selection with good composability: usecases should be obvious.

    Easy to disable unused features.

    Not designed to parse any particular language, but favors those that can avoid backtracking.

# Specific Design decisions

    * Lexer phase

        1. The parser must support filtering. Whitespace, comments, etc. should be easy to filter. This is best done through lexical analysis.

        2. Enables easy span construction and line/column measurement.

        Due to user-specified tokens, the Scanner must be implemented and support cloning. Scanner/Lexer bifurcation is to make it easy to implement a fullly capable lexer.

    * Lexer is passed by owner and supports clone.

        * Alternative: share lexer and maintain state internally. Too complex, too intractable to write combinators for.

        * Alternative: share lexer and clone desired states. Too intractable, safe to clone everything.

        Fortunately, peeking can be used to avoid lexer clones.

    * Useful error messages.

        1. Without constructing spans, error messages are basically useless. The lexer needs to construct spans automatically to simplify error handling.

        2. Error messages should present useful info. Tephra includes pretty parse error formatting.

    * Source text.

        1. Parsable text may have different line and column metrics, as well as file names, which need to be accessible to any errors for formatting. Thus the source text must be wrapped in this information so that it can be extracted automatically anywhere during the parse.

    * Error recovery.

        1. A parser which only emits the first-encoutnered error can be very tedious to work with. Thus the library must support an error sink and automatic recovery.

        2. Error messages will be less useful if they are not able to be handled before being sent to the sink. This necessitates storing "error contexts" to modify these messages before being emitted. Additionally, any non-recoverable error should be processed in the same way, so error contexts should be the primary means of modifying errors.

        3. Error contexts should not be stored in the lexer, because the lexer clone should remain as cheap as possible, but also because the error contexts and lexer contexts are not generally synchronized in practice. (Consider when a lexer is conditinally expected to fail.) Thus the parser functions have to take ownership of the lexer and context separately so they can be managed independently.

        4. Error contexts should be cheap to clone, but they can be expensive to access, because errors are assumed sparse. However, we should be able to completely disable pushing error contexts and accessing them if they would be wasteful because the errors are being silenced.

        5. Error recovery is risky. If a parse fails, any future parses can be assumed invalid via ex falso qoudlibet, so if we never get a good parse after a failure, we can safely assume all of the following errors are a result of the first. An error is only successfully recovered from if there is a successful parse of a specific user-defined value. It should be easy to specify this value and silence any errors encountered before the success.

    * Use boxed error types with downcasting

        1. This allows errors to carry more contextual information and allows for more useful error messages.



## Source span display

### Description
### Source Span
### Highlight Span
### Note / Help
### Color
### Gutter (Line numbers)
### Riser
### Source line
### highlight line
### highlight style (Hat, Dash)
### Span message start
### Span message end


    error[E0308]: mismatched types
      --> src/result/display.rs:60:17
       |
    59 |  /             if hl.start().page.line == line_span.start().page.line {
    60 |  |                 write_source_line(
       |  |_________________^
    61 | ||                     f,
    62 | ||                     gutter_width,
    63 | ||                     highlight_gutter_width,
    64 | ||                     "",
    65 | ||                     "^",
    66 | ||                     "Message")
       | ||______________________________^ expected `()`, found enum `std::result::Result`
    67 |  |             }
       |  |_____________- expected this to be `()`
       |
       = note: expected unit type `()`
                       found enum `std::result::Result<(), std::fmt::Error>`
    help: try adding a semicolon
       |
    66 |                     "Message");
       |                               ^
    help: consider using a semicolon here
       |
    67 |             };
       |              ^

    error: aborting due to previous error

    For more information about this error, try `rustc --explain E0308`.
    error: could not compile `tephra`.



    warning: variable does not need to be mutable
      --> src/result/display.rs:46:9
       |
    46 |     let mut next_hl = hl.next();
       |         ----^^^^^^^
       |         |
       |         help: remove this `mut`
       |
    note: the lint level is defined here
      --> src/lib.rs:32:9
       |
    32 | #![warn(unused)]
       |         ^^^^^^
       = note: `#[warn(unused_mut)]` implied by `#[warn(unused)]`

    warning: 1 warning emitted


# New error & span ownership

Separate references to the source text from the spans. The span operations can be applied to arbitrary text sources, allowing for more code sharing between borrowed & owned structs.

Span (text)
Span
Highlight (span)
HighlightOwned
SpanDisplay (highlight, span)
SpanDisplayOwned
CodeDisplay (span_display)
CodeDisplayOwned
ParseError
ParseErrorOwned

# Tephra Parse Errors



New Error Handling (2023-03-14)
===============================

Three important aspects of nice parsing:
    1. Constructing accurate spans.
    2. Adding context to errors.
    3. Recovering from errors to analyze the rest of the text.


1. Lexer errors are always unexpected tokens. We record which tokens were found and expected.

    Found EOF, expected [something]
    Expected EOF, found [something]
    Expected [something], found [something else]

    Primitive combinators:

    `empty`: parses nothing and returns an empty string.
    `one`: attempts to parse a single token, and returns the text of it's span.
    `any`: attempts to parse a series of tokens, returning the text of the span of the first that succeeds.
    `any_filtered`: parses any filtered tokens, returning the text of the span of the tokens.
    `seq`: attempts to parse a series of tokens, returning the text of the span over all that succeed.
    `end_of_text`: attempts to parse the end of text, returning an empty string if it succeeds.

2. Parse errors are a more general class of errors.

    `fail`: Parses any token and fails.


3. It should always be possible to efficiently 'strip' combinator trees so that minimal context is handled during their parse. This is charactized by the fact that the only errors possible from such a 'stripped' parse are lexer errors. No additional context is provided below the 'strip' point.

    `raw`: Parses a value without collecting context info for errors.

4. Sequential combinators attempt multiple parses and aggregate their results together.

    `left`: parses two values and returns the first.
    `right`: parses two values and returns the second.
    `both`: parses two values and returns the pair.
    `repeat`
    `repeat_count`
    `repeat_until`
    `repeat_count_until`
    `intersperse`
    `intersperse_until`
    `intersperse_count`
    `intersperse_count_until`

5. Recovery combinators handle parse errors by collecting them in a buffer. Unrecoverable errors are passed up the call stack, and the top-level parser should aggregate both the collected and returned errors. Recovery works by identifying a token to parse up to or past. Parsing must continue from where the error is put into the buffer, and an empty result is produced for the parse value. If any errors occur in the parse, the result cannot be valid value, so error aggregation should occur automatically.

    `bracket`
    `bracket_dynamic`
    `delimit`

    Missing delimiter/Unmatched bracket. The process for advancing to the recovery position can be complex if the delimiter is context sensative.


6. Alternative combinators attempt multiple possible parses. The first that succeeds is returned. Errors are suppressed unless an implicative error is returned, or if none of the paths succeed.

    `either`: attempts two parses, returning the first which succeeds.
    `maybe`: attempts a parse which may fail without error (i.e., empty parse alternative.)
    `require_if`: requires a parse to succeed only if the given predicate is true.

    Efficient alternative combinators should use peek to act like implicative combinators.

7. 'Implicative' combinators consist of two parts: A => B. A successful parse of A will require a successful parse of B. Thus it is not a parse error for A to fail, but it is a parse error for B to fail.

    `atomic`: attempts a parse such that if any prefix of it succeeds, the entire parse must succeed.
    `implies`: parses two values and returns the pair if the first succeeds, or `None` if the first fails.
    `cond`: attempts a parse only if the given predicate is true.

9. 'Filter' combinators modify their inputs, but do not interact with error handling, as they modify the input stream before parsing happens.

    `filter_with`: attempts a parse with the given token filter active.
    `unfiltered`: attempts a parse with token filters disabled.

10. 'Transform' combinators modify their outputs, but do not interact with error handling, as they only operate on successful parses.

    `spanned`: wraps the parsed result in its span.
    `discard`: discards the parsed result.
    `text`: replaces the parsed result with the text of its span.

11. General lexing operations:

    * Lexer clone
    * Lexer snip
    * Push context
    * Set context
    * Push error handler
    * Set error handler
    * Lexer restore
    * Advance to
    * Advance past
    * Error recovery on|off

    
# Doc sections

+ Similar combinators
+ Arguments
+ Error recovery
+ Errors
+ Panics


# Tracing targets

+ combinators
+ combinators internals
+ error recovery
+ scanner operations
+ lexer operations
+ span operations
+ context ops
+ results

# Lexer Diagram:


...AAA...BBBCCCDDD...EEE...
      \  \     \  \  \  \
      |  |     |  |  |  peek_cursor
      |  |     |  |  peek_begin
      |  |     |  |
      |  |     |  cursor
      |  |     token_begin
      |  |
      |  parse_begin (synced)
      parse_begin (behind)
      
