# macro-template

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/macro-template.svg
[crates-url]: https://crates.io/crates/macro-template
[docs-badge]: https://img.shields.io/docsrs/macro-template
[docs-url]: https://docs.rs/macro-template
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/macro-template
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/macro-template/workflows/CI/badge.svg
[actions-url]: https://github.com/fast/macro-template/actions?query=workflow%3ACI

<!-- macro-template-docs-start -->

macro-template provides `template!`, a procedural macro for generating repeated Rust code from compact, table-driven inputs.

## Motivation

While working on ScopeDB, I replaced two procedural macros with this crate: [`match-template`](https://github.com/tisonkun/match-template/), which ScopeDB used to generate match arms over rowset concrete types and system-view variants, and [`macro_find_and_replace`](https://github.com/lord-ne/rust-macro-find-and-replace/), which it used for wrappers such as `with_types!` and `with_system_views!` that repeated a block after replacing one identifier with each type in a list.

That migration is the motivating case for `macro-template`. The two old macros looked unrelated at the call site: `match-template` had an assignment-like map syntax such as `T = [...]` or `(Variant, View) = [A => B]`, while `macro_find_and_replace` used positional arguments around the token to replace. But the code I wanted to write had the same shape in both places: bind one or more identifiers to a row of tokens, then expand ordinary Rust tokens with those bindings.

ScopeDB's system views make the shape visible. The useful data is just a table such as `(Databases, DatabasesView)` and `(Schemas, SchemasView)`; the generated code may use one column as an enum variant and the other as a type:

```rust
struct DatabasesView;
struct SchemasView;

impl DatabasesView {
    const TABLE_NAME: &'static str = "databases";
}

impl SchemasView {
    const TABLE_NAME: &'static str = "schemas";
}

enum SystemView {
    Databases(DatabasesView),
    Schemas(SchemasView),
}

fn system_view(table_name: &str) -> Option<SystemView> {
    macro_template::template! {
        for (Variant, View) in [
            (Databases, DatabasesView),
            (Schemas, SchemasView),
        ] {
            match table_name {
                #(View::TABLE_NAME => Some(SystemView::Variant(View)),)*
                _ => None,
            }
        }
    }
}

assert!(matches!(system_view("schemas"), Some(SystemView::Schemas(_))));
```

The problem was not that those crates were bad; it was that every crate had a different mini-language. Reading a call site meant remembering which token was the placeholder, whether `=>` described two substitutions or a match arm, where commas belonged to the macro input, and which part of the Rust body was actually repeated.

I later noticed the same idea in [`seq-macro`](https://github.com/dtolnay/seq-macro): bind an identifier to each number, byte, or character in a range, then expand a Rust fragment, with `#( ... )*` marking the part that repeats inside a surrounding item. That made the common model clearer: this is table-driven token substitution, not a match-specific or sequence-specific trick.

`template!` uses one syntax for these cases:

1. declare one or more template identifiers after `for`;
2. provide rows with `in [...]` or a range with `in 0..N`;
3. write the Rust tokens to generate in the block;
4. add more `for` clauses when independent dimensions should form a Cartesian product.

The goal is not to invent another domain-specific language, but to make the table-driven shape explicit and keep the template body looking like the Rust it will generate.

## Examples

The examples below cover whole-body repetition, partial repetition, ranges, and multi-dimensional inputs.

### Whole-body repetition

Without splice syntax, the whole template body is repeated once per input row:

```rust
trait TypeName {
    const NAME: &'static str;
}

macro_template::template! {
    for (Ty, Name) in [
        (u8, "u8"),
        (u16, "u16"),
        (u32, "u32"),
    ] {
        impl TypeName for Ty {
            const NAME: &'static str = Name;
        }
    }
}

assert_eq!(<u16 as TypeName>::NAME, "u16");
```

### Partial repetition

When only part of a surrounding construct should repeat, put that part in `#( ... )*`. A single separator token tree can be written before `*`, such as `#( ... ),*` for comma-separated output:

```rust
fn keyword_code(text: &str) -> Option<u8> {
    macro_template::template! {
        for (Pat, Code) in [
            ("async", 1u8),
            ("await", 2u8),
        ] {
            match text {
                #(Pat => Some(Code)),*,
                _ => None,
            }
        }
    }
}

assert_eq!(keyword_code("async"), Some(1));
assert_eq!(keyword_code("await"), Some(2));
assert_eq!(keyword_code("fn"), None);
```

When a template contains `#( ... )*` or `#( ... ),*`, template variables are substituted only inside the splice body, and the surrounding tokens are emitted once. Surrounding identifiers stay literal, even when they have the same name as a template variable. If a value should vary, place it in the splice body.

`#( ..., )*` and `#( ... ),*` are different: the latter does not produce a trailing comma. This matches delimiter repetition in `macro_rules!`.

### Range inputs

Inputs can also be ranges of integers, characters, or bytes. Range inputs are written directly after `in`, without surrounding brackets:

```rust
let tuple = (1000, 100, 10);
let mut sum = 0;

macro_template::template! {
    for N in 0..3 {
        sum += tuple.N;
    }
}

assert_eq!(sum, 1110);

let mut chars = String::new();

macro_template::template! {
    for C in 'x'..='z' {
        chars.push(C);
    }
}

assert_eq!(chars, "xyz");
```

Integer ranges preserve the radix, suffix, and shared padding width from their bounds.

### Cartesian products

Multiple input clauses form a Cartesian product in clause order. This is useful when two or more independent dimensions share the same generated body:

```rust
struct Cpu;
struct Gpu;

trait Kernel<T> {
    fn run(input: T) -> T;
}

macro_template::template! {
    for Backend in [Cpu, Gpu],
    for Ty in [f32, f64],
    {
        impl Kernel<Ty> for Backend {
            fn run(input: Ty) -> Ty {
                input
            }
        }
    }
}

assert_eq!(<Gpu as Kernel<f64>>::run(1.5), 1.5);
```

<!-- macro-template-docs-end -->

## Minimum Rust version policy

This crate's minimum supported `rustc` version is `1.85.0`.

The current policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if `crate 1.0` requires Rust 1.85.0, then `crate 1.0.z` for all values of `z` will also require Rust 1.85.0 or newer. However, `crate 1.y` for `y > 0` may require a newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
