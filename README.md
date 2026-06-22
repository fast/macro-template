# macrotable

`macrotable` provides two function-like procedural macros for small
compile-time repetitions over identifier lists.

- `repeat!` emits the whole body once per input row.
- `splice!` emits the body once and expands `#( ... )*` fragments inside it.

## Install

```toml
[dependencies]
macrotable = "0.1"
```

## Example

```rust
use macrotable::{repeat, splice};

#[derive(Debug, PartialEq, Eq)]
enum MetricValue {
    Unsigned(u128),
}

trait IntoMetricValue {
    fn into_metric_value(self) -> MetricValue;
}

repeat!(#T in [u8, u16, u32, u64, usize] {
    impl IntoMetricValue for #T {
        fn into_metric_value(self) -> MetricValue {
            MetricValue::Unsigned(self as u128)
        }
    }
});

struct WorkerStats {
    queued: usize,
    running: usize,
    failed: usize,
}

impl WorkerStats {
    fn counters(&self) -> [(&'static str, usize); 3] {
        splice!(#field in [queued, running, failed] {
            [ #( (stringify!(#field), self.#field) ),* ]
        })
    }
}

fn main() {
    let stats = WorkerStats {
        queued: 4,
        running: 2,
        failed: 1,
    };

    assert_eq!(42u16.into_metric_value(), MetricValue::Unsigned(42));
    assert_eq!(
        stats.counters(),
        [("queued", 4), ("running", 2), ("failed", 1)]
    );
}
```

Run the complete example with:

```sh
cargo run --example metrics
```

## Rules

- Bind placeholders as `#name` and use them as `#name`.
- Input values must be single identifiers. Alias complex types first.
- Tuple rows can bind multiple placeholders, and `_` skips a row value.
- In `splice!`, placeholders from the current invocation are only available
  inside `#( ... )*`.
- Nested invocations are supported. Use different placeholder names at each
  level.

`repeat!` is useful when each repeated block can stand on its own.
`splice!` is useful when repeated pieces must fit inside one surrounding Rust
construct, such as match arms, enum variants, arrays, function arguments, or
macro arguments.

See the crate documentation for full syntax and error behavior.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
