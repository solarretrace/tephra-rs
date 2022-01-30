# Getting Started


Tephra parsers use a combination of builtin types and user-defined types to construct recursive-descent parsers. Different aspects of the parse are religated to different components of the library. The typical process for implementing a full-featured parser looks like this:

1. Define a token type.

2. Define a Scanner type that matches the tokens in text. The scanner should use a ColumnMetrics implementor to measure the tokens.

3. Define a set of parsers which will parse the grammar of your input.

4. If you're defining recursive structures, you may want to implement a set of structure-matchers to simplify the grammar.

5. Create a Lexer from your Scanner and ColumnMetrics, invoke the parser by passing in the lexer, then transform the output into your desired result type.

