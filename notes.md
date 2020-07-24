
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

# Lexer conversions

There's a bit of wasted effort in not doing value production in the lexer. Lexing a number or escaped string is redundant with converting it to the associated value. However, there are several ways to avoid this problem, each with different tradeoffs.

## 1. Non-trival lexer tokens.

The lexer can produce the data in a single pass and emit it with the token. This means token matching becomes more complex, so we'll need to separate Tokens carrying values from TokenTypes, which are used in matching.

## 2. Data in the lexeme.

The lexer always produces a value and stores it in the lexeme. This requires the parser to know which tokens produce values and of which type, so that they may be retrieved. This could involve wrapping low-level parsers in data-extractor combinators, which could be confusing.

## 3. No data conversions in the lexer.

The lexer always produces a simple token and its span. There is some redundant effort in calculating data conversions, but it can be done using normal data conversion combinators. Additionally, if the data is unused (eg., part of a fallible parse,) this may be the most efficient approach.


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

