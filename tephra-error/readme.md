# Tephra Parse Errors


Error recovery & multiple errors
+ Requires storing context from the parse tree to decorate errors during recovery?
+ If so, we may as well store the error list with the recovery context.



    Atomic     = 5,
    Bounded    = 4,
    Delimited  = 3,
    Unbounded  = 2,
    Validation = 1,
    Lexer      = 0,



As errors propagate up through the parser combinators, more contextual information is available to clarify what is wrong.

Errors that pass through contexts, can unify in multiple ways:
    Ignore => The error is replaced with that of the broader context.
    Source => The error is wrapped by the broader context
    Combine => The error is merged with the broader context in an arbitrary way
    Highlight => The error is moved into the broader context as a highlight
    Note => The error is moved into the broader context as a note

Error contexts:

    maybe => Under a maybe parser, an error will never be emitted.
    atomic => Under an atomic parser, an error will only be emitted if some of the parse succeeds.
    section => Under a section parser, an error will always pass through a broader context


maybe -> maybe
atomic -> segment(maybe(mono()))
section -> segment

    context
    mono
    segment
    maybe
    opt

    cut
    opt
    expect
    ensure
    forward
    prefix_commit
    maybe_atomic
    part
    append
    pledge
    rigid
    fixed
    strict
    reserve



New Error Handling (2023-03-14)
===============================

1. Lexer errors are always unexpected tokens. We record which tokens were found and expected.

    Found EOF, expected [something]
    Expected EOF, found [something]
    Expected [something], found [something else]

2. Parse errors are a more general class of errors.

    `fail`: Generates a custom error.


3. It should always be possible to efficiently 'strip' combinator trees so that minimal context is handled during their parse. This is charactized by the fact that the only errors possible from such a 'stripped' parse are lexer errors. No additional context is provided below the 'strip' point.

4. 'Implicative' combinators consist of two parts: A => B

5. 'Iterative' combinators consist of N parts and delimiters.

    `left`: parses two values and returns the first.
    `right`: parses two values and returns the second.
    `both`: parses two values and returns the pair.

    Recoverable?:

    `bracket`: parses three values and returns the second.
    `bracket_dynamic`: parses three values and returns the second, but the third value can depend on the result of the first.



6. 'Filter' combinators modify their inputs, but do not interact with error handling, as they modify the input stream before parsing happens.

7. 'Transform' combinators modify their outputs, but do not interact with error handling, as they only operate on successful parses.

    `spanned`: wraps the parsed result in its span.
    `discard`: discards the parsed result.
    `text`: replaces the parsed result with the text of its span.



