
# Parser design principles
## Use `&'t str`, not `&mut &'t str`.

This makes it easier to back up in case of a failure. 

## Use `std::Result`.
## If a function takes extra args, return a parser.
## If a function takes no extra args, it is the parser.
## Use `FnMut`
## Use `std::error::Error` for failure source.
## Do not box/own all parse errors.
## Impl `PartialEq` on results for testing.
## Return value, token, rest on success.
## Return context, expected, source, and rest on failure.
## Separate lexer and parser.

This allows us to easily filter lexed tokens, i.e., to remove whitespace or comments. It also allows injecting tokens, i.e., to specify indentation levels, or to analyze comments using a separate parser stream. 

Without a dedicated lexer, all intermediate syntactical structure must be filtered or created inline, and it becomes difficult to separate and analyze.

    result
    span
    lexer
    combinator
        text
        token
    primitive
        comment
        float
        integer
        list
        string

# Error Handling

Unexpected eot
    During parse of...
        * Source EOT error
Unrecognized token
    During parse of...
    Skip token and continue lexing?
        * Span of fail
Parsed wrong token
    During parse of...
Error constructing value
    During parse of...
    Which token value...
        * Span of fail



abc

    error: Unexpected end of text.
      --> {source_name?}:LN:COL (bytes N-M)
       |
    LN | [suround] [SPAN OF TOKEN] [suround]
       |            ~~~~~~~~~~~~~~~
    ... During parse of [CONTEXT].

