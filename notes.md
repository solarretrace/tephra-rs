
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
## SourceSpan / Highlight


# Parser design principles
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




# Error Handling

There's a saying in software development called the [ninety-ninety](https://en.wikipedia.org/wiki/Ninety-ninety_rule) rule, and when it comes to designing a quality parser library, it seems that ninety percent of your effort will be spent on trying to create useful error messages. This is a survey of the error handling techniques I've tried and why they don't work.

## Incremental Call Stack

The simplest way to report errors is to simply churn along, and when you encounter a parse which can't proceed, dump the current parse location. The first problem with this is that you have almost no context about what was supposed to be parsed, so this is almost entirely useless. To fix that, you might try to collect a 'stack trace', which seems to help at first. You get error that look something like this:

    parse failure at byte 126: "goodgoodbadbad..."
                                        ^
    .. during a parse of RULE C
    .. during a parse of RULE B
    .. during a parse of RULE A

What you'll probably learn pretty quickly is that is also unhelpful. Sure, you have some idea of where in your code the error occurred, but there are other problems. First, parse rules often attempt multiple things, so it is natural that they fail, and thus you'll only ever be told a rule fails if its last possible attempt fails. Secondly, most of the rules beyond the first listed are useless because they end up telling you vague things like this:

    parse failure at byte 126: "goodgoodbadbad..."
                                        ^
    .. during a parse of expression
    .. during a parse of statement
    .. during a parse of document

... and clearly you can see how unhelpful that is. Often, most of your parses are going to be expressions, statements, documents, and such things. And on the bottom-most level, all you know is that none of the attempted possibilities worked.


## Spans

The next innovation is spans. Instead of just tracking the current parse location, we track spans of text. The first obvious advantage to this is that we'll often have misplaced tokens, and now when an unexpected token occurs, we can highlight the whole thing, giving a clear outline of which token is unexpected. It's a small thing, but it helps. Ideally, we really want the ability to highlight more than a single token though.

This should allow us to get errors that look marginally better:

    parse failure: unexpected token at bytes 126-129: "goodgoodbadbad..."
                                                               ^^^

To do that, we need the ability to join spans together. There are two obvious ways to do this: (1) Every parse emits a span, and then we manually join them to create larger spans; (2) Every parse takes in a span, and attaches any newly parsed tokens to the end automatically. I call (1) explicit joins, and (2) implicit joins.

I recommend starting with the implicit join idea, because more often than not, we want to join spans, and conveniently enough, our lexer has to track the current parse location anyway, so it may as well track the back end of the span and produce the spans we want on demand. This is arguably more efficient as well, because we don't need to actually calculate joined spans after every parse, we just advance the current position (which we would be doing anyway.)

The problem with implicit joins is figuring out how and when to break them up. By default, every span will extend all the way back to the start of the text. Lexer errors are fairly straightforward to fix though:

+ If a token is unexpected, just highlight the span of the last parsed token.

+ If a EOF was encountered, just highlight the current parse position.

Outside of those, there is no obvious and automatic way to determine where spans should break.


## General purpose parsers

Going forward, one of the central constraints is that a given parser shouldn't know anything about its span context -- the amount of span that should be reported if it fails -- unless it is failing with a lexer error. A parser doesn't really know if it parsing an entire document or a small part of one, so parsers should be as general as possible, and as much of the error handling should be pushed as high up in the grammar as possible.

Another big idea is that there are such things as 'bounded' and 'unbounded' parses, determined by whether a parser is effectively recursive or not. It is much easier to push error handling for bounded parsers lower in the grammar because they take on a fewer variety of forms and the details of what was supposed to be present are much clearer. Conversely, unbounded parses can traverse through many intermediate rules, and it is often much less helpful to know what those rules were as opposed to what is bracketting those rules. So in essence, we usually want to phrase errors arrising from failures in unbounded rules in terms of the bounded rules that introduce or bracket them.

Furthermore, in order to keep our parsers general purpose, we want to minimize the amount of special context handling within a parser. For example, the `bracket` combinator is not going to consider any of its arguments as inherently special; it should be suitable for parsing `abc` just as easily as `[b]`.

So every time we're about to introduce an 'open bracket' or delimitted parse, we want to break off into a new span. 


## Recovery

Speaking of brackets, when we have a failure within a delimitted rule, it often makes sense to log an error, advance to the next delimiter, and continue on to look for further errors. 
