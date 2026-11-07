# Numeric types

Numbers use JSON-like decimal syntax: optional leading `-`, integer digits,
optional fractional part, and optional `e`/`E` exponent with optional sign.
Leading zeroes are rejected except for the integer zero; `+` is not allowed at
the beginning; fractional and exponent parts require digits.

Integer tokens are converted to the smallest fitting representation in this
order: negative values use `i64`, then `i128`; nonnegative values use `i64`,
`u64`, `i128`, then `u128`. Out-of-range integers are rejected. Fractional or
exponent notation is parsed as finite `f64`; overflow to infinity is rejected.

NaN and infinities are not CBR numeric literals. `Value::F64` can hold them,
but the formatter rejects non-finite values. Integral floats are formatted
with `.0` to preserve their float interpretation. Negative zero is emitted as
`-0.0` and remains distinct under `Value` total ordering.
