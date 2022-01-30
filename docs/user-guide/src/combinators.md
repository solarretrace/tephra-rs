# Combinators

One of tephra's design goals is to minimize the number of parser combinators supplied. This is to the benefit of the user, as it is far less likely to use the wrong combinator when only a few options are available, and any of the rarer parses that would behave in an exceptional manner immediately stand out due to the use of special handling at the parse site. Moreover it is easier to understand a smaller set of combinators, which makes it that much easier to understand when they are being used incorrectly, or when a manually written parser is doing what a simple combinator would do.

Another consequence of minimizing the set of combinators is that tephra doesn't demand a compositional parser writing style, as would be expected of a parser combinator library in a functional language. It is expected for the user to write imperative code for handling errors and introducing optimizations and tracing into their parsers.

Every combinator provided by tephra is either assumed to be useful for a wide variety of situations, or obvious and trivial in what it does. This is a surprisingly hard standard to meet, so we'll go over each combinator to explain what it is useful for, as well as a broader picture of parsing various common things to show how they are used in practice. We'll also cover some of the control methods available on the parse result types, as they cover some of the functionality that a traditional combinator library would provide.

