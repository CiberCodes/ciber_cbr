# CBR grammar

This document specifies the syntax currently accepted by `ciber_cbr` 0.1.x.
The grammar is expressed in EBNF; lexical constraints are detailed in the
companion pages. `document` is an implicit root object; nested objects use
parentheses.

```ebnf
document       = separators?, [ property, { separator, property }, [ separator ] ], EOF ;
value          = "null" | "true" | "false" | number | string | array | set | object ;
object         = "(", separators?, [ property, { separator, property }, [ separator ] ], ")" ;
array          = "[", separators?, [ value, { separator, value }, [ separator ] ], "]" ;
set            = "{", separators?, [ value, { separator, value }, [ separator ] ], "}" ;
property       = identifier, "=", value ;
separator      = (",", separators?) | newline, { newline }, [ ",", separators? ] ;
separators     = { whitespace | comment | newline } ;
```

Whitespace between tokens is ignored. Newlines can separate properties and
collection entries. A comma may have an optional trailing newline; trailing
commas are accepted. Comments are `//` to end of line and `/* ... */` block
comments. Nested block comments are not supported.

Examples:

```cbr
title = "Example"
active = true
items = [1, 2, 3]
metadata = (owner = "Ciber", retries = 3)
tags = {"stable", "portable"}
```

Invalid examples include `key: 1` (colon is not assignment), `[1 2]` (missing
separator), and `(x = 1` (unclosed object).
