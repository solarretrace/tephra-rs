
# Handling Errors

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

# Additional parse failure contexts
In addition to the above highlightings, it is often beneficial to know what was expected, and more importantly, *why* it was expected. If a parse fails, we know what parser was running and what it is expecting to find. If some prefix of an unambiguous parse succeeds, then we also know why we're expecting something. 

A parse can fail for many reasons, and some of those failures should be ignored depending on context. 

    It is a successful match
    It is an attempt to match
        .. because a prefix of it was successful
            1. Use `atomic` to delineate the section
            2. Why are we parsing?
            3. What failed? -> in ParseError
            4. What was expected. -> in ParseError

        .. because a validation condition failed
            1. use `section` to delineate the section.
            2. Why are we parsing?
            3. What failed? -> in ParseError
            4. What was expected. -> in ParseError
    It is not an attempt to match
        .. because no prefix was successful
            1. use `maybe` to delineate the section
        .. because a validation condition failed
            1. use `maybe` to delineate the section

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


