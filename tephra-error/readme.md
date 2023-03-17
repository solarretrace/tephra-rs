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
    `maybe_if`: requires a parse to succeed only if the given predicate is true.

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

    
