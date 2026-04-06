use silvestre_core::SilvestreImage;

/// Opaque handle to a `SilvestreImage` for use across the FFI boundary.
pub type SilvestreImageHandle = *mut SilvestreImage;

/// Return the version string of the silvestre library.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the program.
#[no_mangle]
pub extern "C" fn silvestre_version() -> *const std::ffi::c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}
