//! SDK for writing [ClickHouse WASM UDFs](https://clickhouse.com/docs/en/sql-reference/functions/wasm_udf)
//! in Rust.
//!
//! # Quick start
//!
//! Create a new crate compiled as a `cdylib`:
//!
//! ```toml
//! # Cargo.toml
//! [package]
//! name = "my_udf"
//! version = "0.1.0"
//! edition = "2024"
//!
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [dependencies]
//! clickhouse-wasm-udf = "0.1"
//! ```
//!
//! Add a `.cargo/config.toml` to build for WASM by default:
//!
//! ```toml
//! # .cargo/config.toml
//! [build]
//! target = "wasm32-unknown-unknown"
//! ```
//!
//! Define your UDF using the [`clickhouse_udf`] attribute macro:
//!
//! ```ignore
//! use clickhouse_wasm_udf::clickhouse_udf;
//!
//! #[clickhouse_udf]
//! fn greet(name: String) -> String {
//!     format!("Hello, {name}!")
//! }
//! ```
//!
//! Build with:
//!
//! ```sh
//! cargo build --release
//! # output: target/wasm32-unknown-unknown/release/my_udf.wasm
//! ```
//!
//! # How it works
//!
//! The [`clickhouse_udf`] macro generates a `#[no_mangle] extern "C"` wrapper
//! that ClickHouse can call. Arguments and return values are serialized with
//! [MessagePack](https://msgpack.org/) — any type implementing
//! [`serde::Serialize`] + [`serde::Deserialize`] is supported: numbers,
//! `String`, `Vec`, `HashMap`, etc.
//!
//! For advanced use cases (e.g. CSV format or zero-copy access to the raw
//! input buffer) you can use [`buffer::RawBuffer`] directly and export the
//! function manually with `#[unsafe(no_mangle)]`.
//!
//! # Error handling
//!
//! Return `Result<T, E>` to propagate errors with the `?` operator.
//! Any error type that implements [`std::fmt::Display`] is accepted — on `Err`
//! the macro calls [`ch_fatal!`] with the error's `Display` message, aborting
//! the call and reporting it to ClickHouse.
//!
//! ```ignore
//! use clickhouse_wasm_udf::clickhouse_udf;
//!
//! #[clickhouse_udf]
//! fn greet(name: String) -> Result<String, String> {
//!     if name.is_empty() {
//!         return Err("Name cannot be empty".into());
//!     }
//!     Ok(format!("Hello, {name}!"))
//! }
//! ```
//!
//! [`ch_log!`] and [`ch_fatal!`] are still available for unconditional logging
//! or aborting outside of the `?` pattern:
//!
//! ```ignore
//! use clickhouse_wasm_udf::{ch_log, ch_fatal, clickhouse_udf};
//!
//! #[clickhouse_udf]
//! fn safe_div(a: f64, b: f64) -> f64 {
//!     if b == 0.0 {
//!         ch_fatal!("division by zero");
//!     }
//!     ch_log!("computing {a} / {b}");
//!     a / b
//! }
//! ```

pub mod buffer;
pub mod host_api;

pub use clickhouse_wasm_udf_macros::clickhouse_udf;

/// Writes a formatted message to the ClickHouse server log.
///
/// Accepts the same format string syntax as [`format!`].
///
/// # Example
///
/// ```ignore
/// ch_log!("processing {} rows", num_rows);
/// ```
#[macro_export]
macro_rules! ch_log {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::host_api::log(&s);
    }}
}

/// Aborts the current UDF call with a formatted error message.
///
/// Accepts the same format string syntax as [`format!`]. The message is
/// reported as an error in ClickHouse. This macro never returns (`-> !`).
///
/// # Example
///
/// ```ignore
/// ch_fatal!("unexpected input: {:?}", value);
/// ```
#[macro_export]
macro_rules! ch_fatal {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        $crate::host_api::fatal(&s)
    }}
}

// Re-exported for use in macro-generated code
#[doc(hidden)]
pub use rmp_serde;
#[doc(hidden)]
pub use serde;
