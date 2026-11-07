# Collections and objects

The root document is an object whose properties are written without
parentheses. Nested objects use `(key = value, ...)`; arrays use `[value, ...]`;
sets use `{value, ...}`. Collections accept comma or newline separators and
trailing commas.

Duplicate object keys are accepted and the last value wins. Objects use
`BTreeMap`, so formatting orders keys lexicographically rather than preserving
insertion order. Sets use `BTreeSet`, discard duplicates and format in the
`Value` ordering. Arrays preserve order and duplicates.

Compact and pretty formatters are deterministic for a given `Value`. Pretty
formatting indents four spaces per nested level. Default parsing limits each
collection/object to 100,000 encountered elements/properties.
