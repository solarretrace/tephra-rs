# Terminology


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

