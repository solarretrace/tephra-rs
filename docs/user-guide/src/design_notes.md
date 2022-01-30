# Tephra Developer Design Notes




## Use `&'t str`, not `&mut &'t str`.

This makes it easier to back up in case of a failure. This only applies to a combined lexer/parser.

Different parser designs may be able to accomodate different data formats. In order to build a streaming parser, the lexer won't be able to return slices into the source, as the lexeme borrows from the lexer's internal buffer, rather than the external source text buffer.


## Use `std::Result`.
## If a function takes extra args, return a parser.
## If a function takes no extra args, it is the parser.
## Use `FnMut`
## Use `std::error::Error` for failure source.
## Do not box/own all parse errors.
## Impl `PartialEq` on results for testing.
## Return value, lexer on success.
## Don't use generics for column metrics.

## Join spans by default, explicitely separate them.

Spans are joined extremely frequently, so it is much simpler to only specify when they should be separated. The lexer should track both its current position and the position of the last unconsumed text. This will enable lexer reuse without cloning, and allow spans to be joined implicitely.
The Lexer::next method returns the span of the most recent token on success, and on error it returns the span of unconsumed text.

## Return lexer, reason, source on failure.

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

## Scanner holds state

The scanner must hold state to allow processing escaped tokens. Nested comments, matching quotes, etc., can be lexed as escaped text or as language tokens, and the scanner needs to know what the open/escape is. So whenever a scanner produces an escape token, it records that state until the corresponding close is produced.

## Scanner outputs Option

If the scanned text is empty, it is obvious that there is no token to scan, and if the scanner returns None, then there is no token to match the text. The main problem with this is in detecting *why* there is no match if text remains, which could be a function of the scanner state.

In practice, this is probably not a problem, because a parser should know whether the scanner is entering such a state, and if the parse fails, we should be able to determine why. This probably means that you can't use simple combinators for e.g., both strings and bracketed tokens, but you would usually want escaped tokens to be processed in their own parsers anyway.

This also means the scanner doesn't need a dedicated error type, and that parse errors arising from the scanner won't need to be boxed, which is simple and more efficient.

## Scan for any token or a specific token?

Scanning for specific tokens would probably be more efficient and will make scanners easier to write. Token patterns can overlap, and the same text can be matched by multiple tokens, depending on what the parser requested.

On the other hand, those ambiguities don't seem relevant in practice. Scanning for any token allows for cleaner iteration, and most importantly, efficiently scanning ahead to a sentinal token. It also allows for more complex filtering capabilities.


# Lexer filtering and span construction

The lexer output should be filterable and contain full-constructed spans before any parser code works on it.

## Lexer trait or lexer struct?

The Lexer could be a trait requiring Iterator over the lexemes. This would allow iterator combinators to do filtering and transformation of the lexer output, as well as allow arbitrary parsers to transform the lexer on demand. On the other hand, it is likely that combinator errors would get difficult to analyze, as the lexer would have many type variables. This also makes it almost impossible to interact with the lexer state during parsing. At a minumum every iterator would need to be clonable so allow backtracking in case of a failed parse.

The lexer could present a struct interface. This is problematic in that it strongly constrains what the lexer is allowed to do. It doesn't allow parsers to transform the lexer without including stateful operations on the lexer. Fortunately, there is not a whole lot that the typical lexer will need to do: filter whitespace, backtrack, push tokens into the stream, ... Most other options can be handled in the parser code.


# Lexer conversions

There's a bit of wasted effort in not doing value production in the lexer. Lexing a number or escaped string is redundant with converting it to the associated value. However, there are several ways to avoid this problem, each with different tradeoffs.

## 1. Non-trival lexer tokens.

The lexer can produce the data in a single pass and emit it with the token. This means token matching becomes more complex, so we'll need to separate Tokens carrying values from TokenTypes, which are used in matching.

## 2. Data in the lexeme.

The lexer always produces a value and stores it in the lexeme. This requires the parser to know which tokens produce values and of which type, so that they may be retrieved. This could involve wrapping low-level parsers in data-extractor combinators, which could be confusing. (And doing it automatically would essentially look the same as #1.)

## 3. No data conversions in the lexer.

The lexer always produces a simple token and its span. There is some redundant effort in calculating data conversions, but it can be done using normal data conversion combinators. Additionally, if the data is unused (eg., part of a fallible parse,) this may be the most efficient approach.

