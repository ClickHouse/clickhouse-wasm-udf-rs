//! Bindings to the ClickHouse host functions available inside a WASM UDF.
//!
//! These functions are provided by the ClickHouse runtime and are only
//! available when running inside a WASM module. Prefer the [`ch_log!`] and
//! [`ch_fatal!`] macros over calling [`log`] and [`fatal`] directly.
//!
//! [`ch_log!`]: crate::ch_log
//! [`ch_fatal!`]: crate::ch_fatal

use std::ffi::c_char;

unsafe extern "C" {
    pub fn clickhouse_log(s: *const c_char, len: usize);
    pub fn clickhouse_throw(s: *const c_char, len: usize) -> !;
    pub fn clickhouse_server_version() -> u64;
    pub fn clickhouse_random(data: *mut u8, size: u32);
}

/// Sends a log message to the ClickHouse server log.
pub fn log(s: &str) {
    unsafe {
        clickhouse_log(s.as_ptr() as *const i8, s.len());
    }
}

/// Aborts the current UDF call and reports `s` as an error to ClickHouse.
pub fn fatal(s: &str) -> ! {
    unsafe {
        clickhouse_throw(s.as_ptr() as *const i8, s.len());
    }
}

/// Returns the ClickHouse server version as a single `u64`.
///
/// The encoding is `${MAJOR} * 1000000 + ${MINOR} * 1000 + ${PATCH}`.
pub fn server_version() -> u64 {
    unsafe { clickhouse_server_version() }
}
