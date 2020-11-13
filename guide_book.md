
# Tephra User's Guide

## Introduction

Tephra is a Rust library for designing parsers.

Tephra began is a simple collection of text-consuming functions, and has evolved over time into a more complex and featurefull library. The following features were gradually introduced according to the reasoning layed out below:

1. Text references are based on `&'t str`, not `&mut &'t str`.

The original design of Tephra used `&mut &'t str`, and had a problem if the parse ever had to return to a prior state, in that you had to make copies of the text reference. The inevitable need to make copies removes any real efficiency or conceptual gains this design might have had, so using shared references is more consistent and simpler overall.

2. Use `std::Result`.

Tephra's parse results are built upon `std::Result`. The hope is that this would simplify extending the library, as one could rely on existing well-understood result methods, but experience with the library indicates that this may not be very valuable; most parser result handling is fairly specialized, even when there are overlaps in functionality, the generic names in `std::Result` don't aid in clarifying the code.

3. Distinguish between parser functions and parser combinators.

As a parser combinator library, it is possible to make primitive parsers into combinators that return parse functions. This leads to a bit of syntax noise in the form of extra function calls (`one(Token)()`, rather than `one(Token)`), but it would make the code more uniform. In practice, parsers are usually invoked implicitely, so it is easier to name the function directly, rather than require it be returned from another function. Combinators are thus only 'invoked' if there are configuration arguments that need to be evaluated.

4. Parsers implement `FnMut`.




### Tokens
### Lexer, Scanner, and ColumnMetrics
### Parsing functions and combinators
### Parse results

## Standard combinators
### Primitive
#### `one`
#### `text`
#### `any` and `seq`

### Join
### Repeat

## Special constructs and design guidelines
### Implementing a parse function.
### Implementing a combinator.
### Understanding `end-of-text`
### Understanding `section`
### Using `Lexer::clone`
### Using `Lexer::sublexer`
### Using `ParseResult::take_value`
### Using `ParseResult::map_value`
### Using `ParseResult::discard_value`
### Using `ParseResult::finish`
