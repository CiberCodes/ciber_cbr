# Compatibility

This is the initial 0.1.x format specification. The accepted grammar and
canonical formatting are those implemented by the corresponding crate
version. Whitespace and comments may be normalized; object key order is
canonicalized; integer widths may change when reparsed while numeric value is
preserved. Consumers should compare parsed values according to these rules,
not source text.

No cross-version compatibility guarantee is made for 0.x releases. Any future
syntax for byte values, non-finite floats, or new escapes requires an explicit
specification and compatibility decision before implementation.
