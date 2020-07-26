
# Parser design principles
## Use `&'t str`, not `&mut &'t str`.

This makes it easier to back up in case of a failure. 

Different parser designs may be able to accomodate different data formats. In order to build a streaming parser, the lexer won't be able to return slices into the source, as the lexeme borrows from the lexer's internal buffer, rather than the external source text buffer.

## Use `std::Result`.
## If a function takes extra args, return a parser.
## If a function takes no extra args, it is the parser.
## Use `FnMut`
## Use `std::error::Error` for failure source.
## Do not box/own all parse errors.
## Impl `PartialEq` on results for testing.
## Return value, token, rest on success.

Implemented as value, span, lexer.

## Return context, expected, source, and rest on failure.

Implemented as reason, source, span, lexer.

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

# Lexer filtering and span construction

The lexer output should be filterable and contain full-constructed spans before any parser code works on it.

## 1. Lexer trait.

The Lexer could be a trait requiring Iterator over the lexemes. This would allow iterator combinators to do filtering and transformation of the lexer output, as well as allow arbitrary parsers to transform the lexer on demand. On the other hand, it is likely that combinator errors would get difficult to analyze, as the lexer would have many type variables. This also makes it almost impossible to interact with the lexer state during parsing. At a minumum every iterator would need to be clonable so allow backtracking in case of a failed parse.

## 2. Lexer struct.

The lexer could present a struct interface. This is problematic in that it strongly constrains what the lexer is allowed to do. It doesn't allow parsers to transform the lexer without including stateful operations on the lexer. Fortunately, there is not a whole lot that the typical lexer will need to do: filter whitespace, backtrack, push tokens into the stream, ... Most other options can be handled in the parser code.


# Lexer conversions

There's a bit of wasted effort in not doing value production in the lexer. Lexing a number or escaped string is redundant with converting it to the associated value. However, there are several ways to avoid this problem, each with different tradeoffs.

## 1. Non-trival lexer tokens.

The lexer can produce the data in a single pass and emit it with the token. This means token matching becomes more complex, so we'll need to separate Tokens carrying values from TokenTypes, which are used in matching.

## 2. Data in the lexeme.

The lexer always produces a value and stores it in the lexeme. This requires the parser to know which tokens produce values and of which type, so that they may be retrieved. This could involve wrapping low-level parsers in data-extractor combinators, which could be confusing. (And doing it automatically would essentially look the same as #1.)

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



a

    error: Unexpected end of text.
      --> {source_name?}:LN:COL (bytes N-M)
       |
    LN | [suround] [SPAN OF TOKEN] [suround]
       |            ~~~~~~~~~~~~~~~
    ... During parse of [CONTEXT].

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

