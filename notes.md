
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

## When to clone the lexer.

Clone the lexer whenever you make and overly-general parse. This will allow `atomic` to work correctly.

## Sections and Recovery

Every time we're about to introduce an 'open bracket' or delimitted parse, we want to break off into a new span. 

Speaking of brackets, when we have a failure within a delimitted rule, it often makes sense to log an error, advance to the next delimiter, and continue on to look for further errors.


## Error relavance and validation

Parsers frequently attempt a sequence of sub-parsers, and perhaps naively, one would expect to swallow up any intermediate errors and move on to try the next thing. This is simple to implement and it works well enough, unless none of the sub-parsers succeed and you 'drop out' at the bottom: at this point, the parse which failed is entirely ambiguous, because they *all* failed.

And frequently, the problem is not due to an obvious grammatical error. All of the correct tokens may be in place, but one of the attempts may have failed in performing some necessary conversion on it, leaving the parser to swallow up that error and go on to do everything else which has no hope of succeeding.

The broader point here is that there is a scaled notion of relevance when multiple sub-parses are performed. In the case that both parses would succeed, the most relevant one is the first to be applied (as one would expect and/or desire for sequential code... If we're parsing concurrently, this question might need a more serious answer.)

If only one of the two succeeds, the successful one is more relevant. However, if both parses fail, the naive process above results in the last one being the most relevant, and that is often *not* what we want, unless none of the rules are gramattically correct. We could try adding a flag to the error type so we can decide whether to return or continue (or perhaps more wildly: simulate "throwing exceptions"), but we should make sure not to handle this decision at the sub-parser's call site, because there may be successful parses following a conversion failure, and we want successful parses to take priority.

Another thing to bear in mind is that grammatical errors only have a few possible error states: unexpected tokens, unexpected end-of-text, unrecognized tokens. All other errors signify that only valid tokens were produced and consumed, so they must be conversion errors.

As an addendum to all of the above, a lot of these kinds of headaches can be avoided by delaying any validation operations as long as possible. You might parse the entire input to validate its grammar and capture all values as text, then do a second pass to convert text values into semantically meaningful values. By doing this, you ensure that all parsers have consistent behavior, because they can only fail due to grammatical errors. The downside of this is that it becomes difficult to compose parsers, because you have to forbid calling into anything that does validation. It also means you can't easily do context-sensitive parsing where you make decisions based on the value of a previous parse, but that's probably something to avoid for other reasons.

# Combinators

One of tephra's design goals is to minimize the number of parser combinators supplied. This is to the benefit of the user, as it is far less likely to use the wrong combinator when only a few options are available, and any of the rarer parses that would behave in an exceptional manner immediately stand out due to the use of special handling at the parse site. Moreover it is easier to understand a smaller set of combinators, which makes it that much easier to understand when they are being used incorrectly, or when a manually written parser is doing what a simple combinator would do.

Another consequence of minimizing the set of combinators is that tephra doesn't demand a compositional parser writing style, as would be expected of a parser combinator library in a functional language. It is expected for the user to write imperative code for handling errors and introducing optimizations and tracing into their parsers.

Every combinator provided by tephra is either assumed to be useful for a wide variety of situations, or obvious and trivial in what it does. This is a surprisingly hard standard to meet, so we'll go over each combinator to explain what it is useful for, as well as a broader picture of parsing various common things to show how they are used in practice. We'll also cover some of the control methods available on the parse result types, as they cover some of the functionality that a traditional combinator library would provide.



## Primitive (Token) Combinators
### `empty`
A mostly useless parser with a trivial implementation. Returns a successful parse of nothing.

### `fail`
### `end_of_text`
A simple and useful combinator. Successfully parses only if there is no more text to process, which is a useful way to ensure everything has been parsed. Often used to validate the final result of all parsing, or as a terminator for a repeating parse.

### `one`
Probably the most useful combinator provided. Constructs a parser the matches a single token. This is usually the first step in any parse.

### `any`
An obvious generalization of `one`, which will successfully parse one of any of the given tokens. Less useful than `one`, because you probably end up matching on the result (but you could also do that with `Lexer::peek` without advancing the lexer state.) This is more useful when you want to dynamically match related tokens, such as when using `bracket_dynamic`.

Example:

In atma-script, this is only used in one place: to dynamically match single and double quotes on strings.

```
let string_close = move |lexer, tok| match tok {
        StringOpenSingle => one(StringCloseSingle)(lexer),
        StringOpenDouble => one(StringCloseDouble)(lexer),
        _ => unreachable!(),
    };

bracket_dynamic(
    any(&[StringOpenSingle, StringOpenDouble]),
    text(one(StringText)),
    string_close)
```

Without `any`, the solution would be to attempt each parse seperately, or to write the implementations of these manually.

### `seq`
Another obvious generalization of `one`, which will successfully parse the given sequence of tokens. This is often used with `exact`, but in many situations, a proper `Scanner` impl will obviate the need for this.

Example:

In atma-script, this is only used once to parse color hex codes. because the `HexDigits` token overlaps with identifiers, the `Hash` token changes the scanning mode to avoid parsing idents until a `HexDigits` is found.

```
text(exact(seq(&[
    AtmaToken::Hash,
    AtmaToken::HexDigits])))
```

Alternatively, the Scanner could just consume the `Hash` and `HexDigits` as a single token. So it ends up being a very minor trade-off between scanner complexity and parser complexity.

## Control Combinators
### `filter_with`
This combinator is not very useful, but it is a more general form of the sometimes-useful `exact` combinator. It allows you to temporarily set the lexer filters during a parse so that you can selectively ignore tokens.

### `exact`
This combinator is occasionally useful, but the implementation is quite general. It disables all lexer filters during a parse, allowing you to take manual control over the exact tokens consumed. This is useful if you want to do something like parse two tokens without any whitespace between them. (A potentially preferred alternative to this would to redesign the `Scanner` to match the combined tokens as a new token.)

### `section`
### `discard`
A mostly useless parser with a trivial implementation. Discards the parse result and converts it to `()`.

### `text`
This combinator substitutes a successful parse result with the text it was parsed from. This is commonly wrapped around token combinators to extract the text of the token that was matched.

### `spanned`
This should be one of your most-used combinators. It captures the span of a parse, and is incredibly useful for describing errors from a parse.

## Join Combinators
### `left`, `right`, `both`, `bracket`
These are common and useful combinators that allow you to perform a sequence of parses, capturing specific parts of the output. Often used to parse prefixes, postfixes, delimiters, brackets, etc.

### `bracket_dynamic`
This is an occasionally useful generalization of `bracket`, that passes the output of the first parser to the third parser. This can be used to match closing brackets with opening brackets. When the bracket is a token, it often makes more sense to push a token context in the `Scanner` to ensure the correct ending token is encountered (such as with comments or strings.)

## Repeat Combinators
### `repeat`
### `repeat_until`
### `repeat_collect`
### `repeat_collect_until`
### `intersperse`
### `intersperse_until`
### `intersperse_collect`
### `intersperse_collect_until`
## Option Combinators
### `maybe`

This combinator is sometimes useful when parsing something that is entirely optional. They key word being 'entirely': if any part of the parse suceeds, but not the entire parse, this will succeed with `None`. The behavior of `atomic` is often more useful, but `maybe` is simpler and makes sense for smaller parses, such as looking for extra optional tokens. 

Example:

In StmaScript, trailing commas are accepted in array expressions. The `maybe` combinator allows the parse to tolerate this extra bit of notation:

```
bracket(
    one(OpenBracket),
    left(
        intersperse_collect(0, None,
            expr,
            one(Comma)),
        maybe(one(Comma))),
    one(CloseBracket))
    (lexer)
```


### `atomic`

This combinator is useful to attempt a parse when the parse call tree is determined by a prefix. Basically, as soon as any part of the given parser succeeds, it is an error for the remainder of the parse to fail.

The `atomic` combinator can sometimes cause headaches if you improperly clone (or fail to clone) the lexer. For example, if you parse a more general token at the beginning of the parse and fail if it doesn't satisfy some constraint (such parsing a identifier and succeeding only if it matches a specific keyword), then the `atomic` wrapper will make this parse fail if *any* version of the general parse is successful (so it will fail on any identifier match.) To avoid this, you always want to clone the lexer before making more general parses than necessary, and then fail by returning the original.


### `require_if`
