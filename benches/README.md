# Benchmarks

Run the dependency-free benchmark harness with:

```sh
cargo bench --bench cbr_bench
```

The harness reports total elapsed time and average time per iteration. It
covers parsing 500 ASCII properties, Unicode and 64 KiB strings, nesting at
depth 64, and compact/pretty formatting of a 10,000-element array. Parsing and
formatting iterations are repeated (200 and 100 respectively) with
`std::hint::black_box` to reduce dead-code elimination.

These numbers are comparative measurements, not statistically rigorous
confidence intervals. Run on an otherwise idle machine, record Rust version,
CPU, operating system, and command when comparing revisions. The lexer
materializes a `Vec<char>`. The formatter writer APIs stream directly to their
destination. The harness tracks peak live allocation bytes, but not allocator
fragmentation or resident set size. No performance claim should be inferred
from a single run.

## Reference run

One run in the project container with `rustc 1.100.0-nightly` on
`x86_64-unknown-linux-gnu` produced:

| Workload | Iterations | Average |
| --- | ---: | ---: |
| Parse, 500 ASCII properties | 200 | 256.294 µs |
| Parse, Unicode string | 200 | 38.141 µs |
| Parse, 64 KiB string | 200 | 312.505 µs |
| Parse, nesting depth 64 | 200 | 9.179 µs |
| Format, 10k array compact | 100 | 303.729 µs |
| Format, 10k array pretty | 100 | 393.775 µs |

The tracking allocator reported 65,536 bytes peak additional live heap for
`to_string` on the 10k array, 0 bytes when streaming the same value into
`std::io::sink()`, and 4,395,908 bytes for parsing 10k properties. These are
requested live allocation bytes in this harness, not RSS or total process
memory.

This is a smoke-test reference, not a stable baseline: hardware, system load,
and compiler revision affect timings. No claim should be inferred from the
comparison with a previous run.
