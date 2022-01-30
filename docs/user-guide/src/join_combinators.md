# Join Combinators


### `left`, `right`, `both`, `bracket`
These are common and useful combinators that allow you to perform a sequence of parses, capturing specific parts of the output. Often used to parse prefixes, postfixes, delimiters, brackets, etc.

### `bracket_dynamic`
This is an occasionally useful generalization of `bracket`, that passes the output of the first parser to the third parser. This can be used to match closing brackets with opening brackets. When the bracket is a token, it often makes more sense to push a token context in the `Scanner` to ensure the correct ending token is encountered (such as with comments or strings.)

