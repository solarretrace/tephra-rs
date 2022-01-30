# Section Combinators

### `section`
### `spanned`
This should be one of your most-used combinators. It captures the span of a parse, and is incredibly useful for describing errors from a parse.

### `atomic`

This combinator is useful to attempt a parse when the parse call tree is determined by a prefix. Basically, as soon as any part of the given parser succeeds, it is an error for the remainder of the parse to fail.

The `atomic` combinator can sometimes cause headaches if you improperly clone (or fail to clone) the lexer. For example, if you parse a more general token at the beginning of the parse and fail if it doesn't satisfy some constraint (such parsing a identifier and succeeding only if it matches a specific keyword), then the `atomic` wrapper will make this parse fail if *any* version of the general parse is successful (so it will fail on any identifier match.) To avoid this, you always want to clone the lexer before making more general parses than necessary, and then fail by returning the original.

