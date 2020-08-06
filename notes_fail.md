

# Tephra Error Handling

If a parse succeeds, it is assumed that lexer it returns is consumed up to current. However, if the overall parse is not yet complete, then it should save its own lexer in which the child parse was not yet consumed. If there is any possibility of a parse failure, we want all of the unconsumed text to be available.

`Success` always returns the value and the span it was derived from, and a lexer which is ready to consume more.

`Failure` returns the lexer that is ready to try a different parse instead, and a span up to and over the exact token which failed.

Each parser much satisfy these rules, usually by cloning the lexer. A combinator thus usually has two lexers, one which is cloned to start, and one which is passed to each subparser and extracted from its result. In the case of success, the passed along lexer should be returned after consuming all input. In the case of failure, the passed along lexer will contain the subparser's failure state, which can be packaged up with the cloned lexer as a broader failure context.


# Failure Modes

## Unexpected end-of-text

Unexpected end of text is a parser error. It is generally only constructed in primitives, and occurs when a parser requests a token and the lexer returns nothing.

If the error occurs during a parse, it can helpful to know where the parse started, e.g., to know if it started with a bracket that was unclosed. Parsers expecting bracket tokens should interpret end-of-text as a missing bracket and highlight the unmatched pair.


## Unmatched quote or comment

Unmatched quotes or comments are lexer? errors. Because quotes and comments are general escaped syntaxes that can contain anything, there's not much use in searching for a continuation point after one starts.


Is it necessary that the error be created by the lexer? The lexer could just emit the remainder of the text as a token and let the parser handle the unmatched quote as an end-of-text error.

## Unrecognized Token

Unrecognized tokens are lexer errors. The lexer can't know how much further to process to consume a complete token, but it can advance by character until a recognized token is found.

If the error occurs during a parse, the whole parse is probably wrong, and it doesn't much matter to know where the parse started. It is much more important to know what state the scanner is in.

## Unexpected Token

Unexpected tokens are parser errors. Probably the most common error, it occurs when the parser requests a specific token and receives a different one. These are commonly skipped and another parse is attempted.

If the token is really an error, it is helpful to know where the parse started.

## Unmatched bracket

Unmatched brackets are a parser error.

## Data conversion error

Data conversion errors occur after a portion of a parse has succeeded and the spanned text fails to convert into the parse value.

If the conversion fails, it is helpful to know exactly what text was used in the conversion, so parse conversions should usually happen as early as possible.



# Failure data requirements

All errors that occur during a parse must have assigned spans and descriptions.

## Description

A general description of the type of error that occurred.

## Source

Another error which is the cause of the error.

## Span / Lexer

The lexical extent to which the error applies.

## Span message

A description to apply to the start or end of the span to show how the general desciption applies in this specific case

## Notes

Additional information which might make the error clearer.

## Help

Information which might help in fixing the error.

