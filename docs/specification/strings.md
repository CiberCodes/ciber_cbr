# Strings

Strings are UTF-8 text enclosed in double quotes. The only accepted escapes
are quote, backslash, line feed (`\n`), carriage return (`\r`), tab (`\t`),
and NUL (`\0`). Raw line breaks are forbidden inside a string. Other Unicode
characters are preserved without normalization.

The parser's default decoded string limit is 1 MiB, measured in UTF-8 bytes;
`ParseLimits` can change this. The formatter rejects unsupported control
characters rather than emitting an ambiguous escape.
