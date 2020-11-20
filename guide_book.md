
# Tephra User's Guide

## Introduction


### Design Goals

Tephra is a Rust library for designing parsers.

Tephra began is a simple collection of text-consuming functions, and has evolved over time into a more complex and featurefull library. The following features were gradually introduced according to the reasoning layed out below:

1. Text references are based on `&'t str`, not `&mut &'t str`.

The original design of Tephra used `&mut &'t str`, and had a problem if the parse ever had to return to a prior state, in that you had to make copies of the text reference. The inevitable need to make copies removes any real efficiency or conceptual gains this design might have had, so using shared references is more consistent and simpler overall.

2. Use `std::Result`.

Tephra's parse results are built upon `std::Result`. The hope is that this would simplify extending the library, as one could rely on existing well-understood result methods, but experience with the library indicates that this may not be very valuable; most parser result handling is fairly specialized, even when there are overlaps in functionality, the generic names in `std::Result` don't aid in clarifying the code.

3. Distinguish between parser functions and parser combinators.

As a parser combinator library, it is possible to make primitive parsers into combinators that return parse functions. This leads to a bit of syntax noise in the form of extra function calls (`one(Token)()`, rather than `one(Token)`), but it would make the code more uniform. In practice, parsers are usually invoked implicitely, so it is easier to name the function directly, rather than require it be returned from another function. Combinators are thus only 'invoked' if there are configuration arguments that need to be evaluated.

4. Parsers implement `FnMut`.

5. Use separate lexer and parser phases.

### Parsing strategy

Tephra parsers use a combination of builtin types and user-defined types to construct recursive-descent parsers. Different aspects of the parse are religated to different components of the library. The typical process for implementing a full-featured parser looks like this:

1. Define a token type.

2. Define a Scanner type that matches the tokens in text. The scanner should use a ColumnMetrics implementor to measure the tokens.

3. Define a set of parsers which will parse the grammar of your input.

4. If you're defining recursive structures, you may want to implement a set of structure-matchers to simplify the grammar.

5. Create a Lexer from your Scanner and ColumnMetrics, invoke the parser by passing in the lexer, then transform the output into your desired result type.


### Tokens

Tokens are emitted by the Lexer and Scanner impls, and are defined according to the associated type on Scanner:

    type Token: Display + Debug + Clone + PartialEq + Send + Sync + 'static;

The `Display` impl should be what is shown for a token in error messages, while the `Debug` impl is what appears in traces.

### Lexer, Scanner, and ColumnMetrics

In order to begin parsing with tephra, a `Lexer` must be created. The `Lexer` is a builtin struct that wraps a `Scanner` impl with the appropriate `Token` type. The `Lexer` extends the `Scanner` by holding the source text, column metrics, and automatically creating spans and filtering tokens.

The `Lexer::next` method will invoke `Scanner::next`, which is expected to produce the next scanned token and a measure of how far to advance any spans that include it. To properly implement a `Scanner`, one should use the provided `ColumnMetrics` to compute span advancements (`Pos`), and any scanning state should be internal to the `Scanner`, such as nesting depth or delimitted context flags. It may be worthwhile to track additional scanning state to make the scanner more efficient, e.g., by changing the priority of checking for different tokens based on what was previously scanned.


### Parsing functions and combinators

Parsers are functions which take in a `Lexer` and return a `ParseResult`. Parser combinators take in a set of configuration arguments and return a parser closure. Parsers and combinators should be generic over the `ColumnMetrics` and `Scanner` implementations, and additionally any sub-parsers or result types returned by them.

Tephra provides a suite of primitive parsers and combinators which are suitable for composing into more complicated parsers.

### Parse results

The `ParseResult` type returned by a parser is an alias for `Result<Success<...>, Failure<...>>`, which is generic over the lifetime of the source text, the `ColumnMetrics` and `Scanner`, and the value created by a successful parse. The `Success` type holds the parsed value and the `Lexer` state after finishing the parse. The `Failure` type holds the error value and the `Lexer` state after failing the parse. In a typical scenerio, the lexer is passed into a parser and extracted from its result to continue a sequence of parses. To parse alternatives instead of sequences, it makes more sense to clone the lexer, pass it into each possibility, and return the lexer of the option that succeeds.


## Standard combinators

The builtin tephra combinators are designed for generality, and do not provide any contextual error wrapping. This means that, if one builds a parser using a combination of tephra primitives, that the exact sequence of primitives used will not be deducable from the returned error output. However, the primitives each implement a context span for tracing, so that the primitive sequence can be debugged effectively.

### Builtin Parsers and Combinators

#### `empty` and `fail`

#### `one`

The `one` combinator is straightforward: provide it with a `Scanner::Token` argument, and it will return a parser which succeeds only if the scanned token equals the given token (via `PartialEq`). The value of the result is the scanned token.

#### `any` and `seq`

#### `text`

The `text` combinator simply substitutes the result of a parse for the text in the span of the parse. If token filtering is active, the filtered text will be omitted from the result.

#### `exact`
#### `end_of_text`
#### `spanned`
#### `discard`
#### `filter`

#### `left`
#### `right`
#### `both`
#### `bracket`
#### `bracket_symmetric`
#### `bracket_dynamic`

#### `maybe` and `atomic`
#### `require_if`

#### `repeat`
#### `repeat_collect`
#### `repeat_until`
#### `repeat_collect_until`
#### `intersperse`
#### `intersperse_collect`
#### `intersperse_until`
#### `intersperse_collect_until`



## Special constructs and design guidelines
### Implementing a parse function.
### Implementing a combinator.
### Understanding `end-of-text`
### Using `ParseResult::take_value`
### Using `ParseResult::map_value`
### Using `ParseResult::discard_value`
### Using `ParseResult::finish`


### Using `Lexer::clone` for backtracking
### Using `Lexer::peek` for efficiency
### Error handling
### Lexer errors and Parser errors


## Using SourceDisplay
