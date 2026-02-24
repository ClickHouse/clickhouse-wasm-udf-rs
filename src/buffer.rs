//! Raw byte buffer shared between the WASM module and the ClickHouse host.
//!
//! ClickHouse allocates and frees memory inside the WASM module using the
//! exported [`clickhouse_create_buffer`] and [`clickhouse_destroy_buffer`]
//! functions, which are automatically included when you depend on this crate.
//!
//! You only need to interact with [`RawBuffer`] directly when implementing a
//! UDF manually (without the [`clickhouse_udf`](crate::clickhouse_udf) macro)

/// A contiguous byte buffer whose memory is owned by this WASM module.
#[repr(C)]
#[derive(Debug, Default)]
pub struct RawBuffer {
    pub ptr: *const u8,
    pub len: usize,
    pub capacity: usize,
}

impl RawBuffer {
    /// Converts a `Vec<u8>` into a `RawBuffer`, transferring ownership of the
    /// heap allocation to the caller. The memory must eventually be freed by
    /// passing the enclosing `*mut RawBuffer` to [`clickhouse_destroy_buffer`].
    pub fn from_vec(v: Vec<u8>) -> Self {
        let ptr = v.as_ptr();
        let len = v.len();
        let capacity = v.capacity();
        std::mem::forget(v);
        RawBuffer { ptr, len, capacity }
    }
}

/// Allocates a zeroed buffer of `size` bytes inside the WASM module.
///
/// Called by the ClickHouse host to prepare input data.
#[unsafe(no_mangle)]
pub extern "C" fn clickhouse_create_buffer(size: usize) -> *mut RawBuffer {
    if size == 0 {
        return std::ptr::null_mut();
    }
    let buf = RawBuffer::from_vec(vec![0; size]);
    Box::into_raw(Box::new(buf))
}

/// Frees a buffer previously allocated by [`clickhouse_create_buffer`].
///
/// Called by the ClickHouse host after it has finished reading the UDF output.
#[unsafe(no_mangle)]
pub extern "C" fn clickhouse_destroy_buffer(ptr: *mut RawBuffer) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let buf = Box::from_raw(ptr);
        let _ = Vec::<u8>::from_raw_parts(buf.ptr as *mut u8, buf.len, buf.capacity);
    }
}
