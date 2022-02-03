
# Handling Errors

There's a saying in software development called the [ninety-ninety](https://en.wikipedia.org/wiki/Ninety-ninety_rule) rule, and when it comes to designing a quality parser library, it seems that ninety percent of our effort will be spent on trying to create useful error messages. This is a survey of the error handling techniques I've tried and why they don't work.


## Simple Error Handling

The simplest way to report errors is to simply churn along, and when we encounter a parse which can't proceed, dump the current parse location. The first problem with this is that we have almost no context about what was supposed to be parsed, so this is almost entirely useless.

    parse failure at byte 126: "goodgoodbadbad..."
                                        ^
    .. expected something good, found something bad

This is an incredibly frustrating error, because we don't know *why* the parser was expecting 'good', so we don't know whether our input is wrong or the parser is wrong, and we have no idea *where* either of these might be wrong.

We'll call these kinds of 'no-context' errors "Lexer Errors", because they are the direct result of encountering the wrong token in the lexer. There are four kinds of lexer errors:

+ Unexpected end-of-text
+ Unexpected text (AKA Expected end-of-text, AKA Unexpected non-end-of-text)
+ Invalid Token
+ Unexpected Token

What we ultimately want to do with these is ignore them by trying alternative parses, or add meaningful context to them when they pass back up to the user through the parse tree. If the users are seeing these errors directly, we'll usually create a poor experience.


## Stack Trace Error Handling

To add context to the errors, we might try to collect a 'stack trace', which seems to help at first. We then get errors that look something like this:

    parse failure at byte 126: "goodgoodbadbad..."
                                        ^
    .. expected something good, found something bad
    .. during a parse of RULE C
    .. during a parse of RULE B
    .. during a parse of RULE A

What we'll quickly learn is that this is also usually unhelpful. Sure, we have some idea of where in our code the error occurred, but there are other problems. First, parse rules often attempt multiple alternatives, so it is natural that they fail, and with this, we'll only ever be shown an error on the last alternative tried. Secondly, most of the rules beyond the first listed are useless because they end up telling we vague things like this:

    parse failure at byte 126: "goodgoodbadbad..."
                                        ^
    .. expected something good, found something bad
    .. during a parse of expression
    .. during a parse of statement
    .. during a parse of document

... which is unhelpful because it's only telling us what we what we already know, which is that certain patterns can appear in certain contexts. Often, most of our parses are going to be expressions, statements, documents, and such things. And in the end, all we really learn is that none of the attempted possibilities worked.


## Error Handling with Spans

The next innovation is spans. Instead of just tracking the current parse location, we'll track spans of text, so we can tell where the current parse began. The first obvious advantage to this is that we'll have a better idea of what tokens are being lexed. Now when an unexpected token occurs, we can highlight the entire token, giving a clear outline of which token is unexpected. It's a small thing, but it helps.

This should allow us to get errors that look marginally better:

    parse failure: unexpected token at bytes 126-129: "goodgoodbadbad..."
                                                               ^^^ bad thing here
    .. expected something good, found something bad

Ideally, we really want the ability to highlight more than a single token, and to do that, we need the ability to join spans together. There are two obvious ways to do this: (1) Every parse emits a span, and then we manually join them to create larger spans; (2) Every parse takes in a span, and attaches any newly parsed tokens to the end of it automatically. I'll call (1) explicit joins, and (2) implicit joins.

I recommend starting with the implicit join idea, because more often than not, we want to join spans, and conveniently enough, our lexer has to track the current parse location anyway, so it may as well track the back end of the span and produce the spans we want on demand. This is arguably more efficient as well, because we don't need to actually calculate joined spans after every parse, we just advance the current position (which we have to do regardless.)

The problem with implicit joins is figuring out how and when to break them up. By default, every span will extend all the way back to the start of the text. Lexer errors are fairly straightforward to handle though:

+ If a token is unexpected, we highlight the span of the last parsed token.
+ If a EOF was encountered, we highlight the current parse position.

Outside of those, there is no obvious and automatic way to determine where spans should break.

## User-defined Contexts

Due to our inability to automatically break spans, we will require the user to manually break spans (hopefully without too much ceremony.) One option is to have the user cut spans directly in the lexer. Another option, which will happen quite often, will involve cloning the lexer as well as cutting the span in order to maintain the lex position for other parse alternatives. We'll call the combined clone + cut operation a 'sublexer' clone. A third option we'll often want is to add contextual information to a parse at the same time we cut the span, and we'll call this operation a `context` parse.


## Committed Alternatives

Another great idea is understanding when we've become committed to a particular branch in our parse tree. This happens when we've successfully parsed a unique prefix of one of an alternative set of parse options. Once we've done this, we know that no other alternative is valid, so any parse errors must be due to a failure to finish the alternative we're committed to.

This allows us to attach context to specific alternatives, rather than to the group as a whole. This is particularly valuable when there are a large number of alternatives, because we avoid vague errors that say this:

    "expected one of option A, option B, option C, ... option Z, found something bad"

.. and replace them with errors that say this:

    "found something bad during parse of option D."

But we can only achieve this if we have some model of becoming committed to this alternative. Moreover, the implementation of this has to take into account token filtering. It would be a mistake to become committed to a branch just because we've successfully parsed a comment, for instance.


## Error Collection

All of the above analysis assumes that it is sufficient to report a single parse error for a given document. In practice, it can get very tedious to perform a full parse for every error, especially when errors can be a consequence of other errors. So we really want the ability to collect multiple errors during a parse, which means recovering from a parse failure by finding somewhere to continue the parse.

Error collection introduces an interesting design decision for the parsing library. Do we want to return lists of errors and join them, or do we want to maintain a handle to an external error collector? The choice might seem arbitrary, but much like the case with our decision to automate span construction in the lexer, the work we already need may ends up deciding for us.

The main problems to be solved by error recovery are (1) where in the text do we continue, and (2) what kind of parse to perform there. The first problem is not something with any automatic solution, because it depends on the structure of the language. However, the second has the obvious solution of continuing with the next parse after that which failed. This is probably an ill-defined notion except in the case where our parse is bracketed, postfixed, prefixed, or otherwise delimitted in some manner. Then we can scan ahead for the delimitter and continue from there.

A secondary problem is that of constructing the errors with the appropriate context information. Up until now, we've been assuming that errors will get wrapped in contextual information as they bubble up the call stack. This means we either need to continue to allow all errors to bubble up to the root (and subsequently find a way to return back down to the parse context where we intend to recover to,) or instead store the parse contexts on the way down so that we can recover in place by wrapping the errors in that context before emitting them. The former would basically require first-class [continuations](https://en.wikipedia.org/wiki/Continuation). The latter only requires us to build & store a stack of context information, which incedentally plays nicely with the plan to store a handle to an external error collector.



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
