
# Terminology

## Parser
## Combinator / Primitive
## Lexer
## Scanner
## Token
## NewLine
## Span
## Position / Pos
## Source
## ParseResult / Success / Failure
## OwnedFailure / OwnedSpan
## SourceSpan / Highlight


# Parser design principles

# Error Handling

Unexpected eot
    During parse of...
        * Source EOT error
Unrecognized token
    During parse of...
    Skip token and continue lexing?
        * Span of fail
Parsed wrong token
    During parse of...
Error constructing value
    During parse of...
    Which token value...
        * Span of fail



## Source span display

### Description
### Source Span
### Highlight Span
### Note / Help
### Color
### Gutter (Line numbers)
### Riser
### Source line
### highlight line
### highlight style (Hat, Dash)
### Span message start
### Span message end


a

    error[E0308]: mismatched types
      --> src/result/display.rs:60:17
       |
    59 |  /             if hl.start().page.line == line_span.start().page.line {
    60 |  |                 write_source_line(
       |  |_________________^
    61 | ||                     f,
    62 | ||                     gutter_width,
    63 | ||                     highlight_gutter_width,
    64 | ||                     "",
    65 | ||                     "^",
    66 | ||                     "Message")
       | ||______________________________^ expected `()`, found enum `std::result::Result`
    67 |  |             }
       |  |_____________- expected this to be `()`
       |
       = note: expected unit type `()`
                       found enum `std::result::Result<(), std::fmt::Error>`
    help: try adding a semicolon
       |
    66 |                     "Message");
       |                               ^
    help: consider using a semicolon here
       |
    67 |             };
       |              ^

    error: aborting due to previous error

    For more information about this error, try `rustc --explain E0308`.
    error: could not compile `tephra`.




    warning: variable does not need to be mutable
      --> src/result/display.rs:46:9
       |
    46 |     let mut next_hl = hl.next();
       |         ----^^^^^^^
       |         |
       |         help: remove this `mut`
       |
    note: the lint level is defined here
      --> src/lib.rs:32:9
       |
    32 | #![warn(unused)]
       |         ^^^^^^
       = note: `#[warn(unused_mut)]` implied by `#[warn(unused)]`

    warning: 1 warning emitted



