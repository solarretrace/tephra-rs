# Repetition Combinators


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
