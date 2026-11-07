# Lexical rules

The lexer recognizes identifiers, strings, numbers, booleans, `null`, braces,
brackets, parentheses, `=`, comma, newline and EOF. Token positions are
one-based line/column positions. CRLF is one line break; lone CR and LF also
advance the line.

Identifiers begin with `_` or a Unicode alphabetic character. Remaining
characters may be `_`, Unicode alphanumeric characters, or `-`; an identifier
cannot end in `-`. `true`, `false`, and `null` are reserved literals.

Horizontal Unicode whitespace is ignored. Newlines remain separator tokens.
`//` comments stop before the newline. `/* ... */` comments may span lines and
must terminate. A slash not starting a comment is invalid.

Strings are double quoted, cannot contain literal CR or LF, and support these
escapes: `\"`, `\\`, `\n`, `\r`, `\t`, and `\0`. Other escapes are errors.
String contents are UTF-8.
