# Control Combinators

## Control Combinators
### `filter_with`
This combinator is not very useful, but it is a more general form of the sometimes-useful `exact` combinator. It allows you to temporarily set the lexer filters during a parse so that you can selectively ignore tokens.

### `exact`
This combinator is occasionally useful, but the implementation is quite general. It disables all lexer filters during a parse, allowing you to take manual control over the exact tokens consumed. This is useful if you want to do something like parse two tokens without any whitespace between them. (A potentially preferred alternative to this would to redesign the `Scanner` to match the combined tokens as a new token.)
### `discard`
A mostly useless parser with a trivial implementation. Discards the parse result and converts it to `()`.

### `text`
This combinator substitutes a successful parse result with the text it was parsed from. This is commonly wrapped around token combinators to extract the text of the token that was matched.

### `require_if`
