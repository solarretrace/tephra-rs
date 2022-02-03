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
