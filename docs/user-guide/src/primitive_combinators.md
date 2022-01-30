# Primitive Combinators


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
