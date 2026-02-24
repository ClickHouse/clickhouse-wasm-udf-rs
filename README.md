# clickhouse-wasm-udf

Rust SDK for writing [ClickHouse WASM UDFs](https://clickhouse.com/docs/en/sql-reference/functions/wasm_udf).

## Setup

Create a new library crate:

```sh
cargo new --lib my_udf
```

Configure `Cargo.toml`:

```toml
[package]
name = "my_udf"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
clickhouse-wasm-udf = "0.1"
```

Set the default build target in `.cargo/config.toml`:

```toml
[build]
target = "wasm32-unknown-unknown"
```

## Writing a UDF

Use the `#[clickhouse_udf]` attribute macro. Arguments and return values can be
any type that implements `serde::Serialize` + `serde::Deserialize` — numbers,
`String`, `Vec`, `HashMap`, etc.

```rust
use clickhouse_wasm_udf::clickhouse_udf;

#[clickhouse_udf]
fn greet(name: String) -> Result<String, String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    Ok(format!("Hello, {name}!"))
}
```

### Logging and errors

```rust
use clickhouse_wasm_udf::{ch_log, ch_fatal, clickhouse_udf};

#[clickhouse_udf]
fn safe_div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        ch_fatal!("division by zero");
    }
    ch_log!("computing {a} / {b}");
    a / b
}
```

`ch_log!` writes to the ClickHouse server log. `ch_fatal!` aborts the call and
reports an error.

## Building

```sh
cargo build --release
# output: target/wasm32-unknown-unknown/release/my_udf.wasm
```
