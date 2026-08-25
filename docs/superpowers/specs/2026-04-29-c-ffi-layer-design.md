# C FFI Layer & Foreign Language Bindings Design Specification

**Date:** 2026-04-29  
**Topic:** C-Compatible ABI, Opaque Pointer Lifecycles, and Header Generation  
**Pull Requests:** #59  
**Status:** Implemented  

---

## 1. Overview

This specification defines the C Foreign Function Interface (FFI) layer `silvestre-ffi`, enabling integration of Silvestre image processing algorithms into C, C++, Swift, Objective-C, and Android JNI native codebases.

---

## 2. ABI Design & Memory Management

### 2.1 C-Compatible API
- All public functions are marked `#[no_mangle]` with `extern "C"` linkage.
- The crate builds both dynamic library (`cdylib`) and static archive (`staticlib`) targets.

### 2.2 Opaque Handle Pattern
To ensure memory safety and prevent undefined behavior, Rust owns the underlying pixel allocations. Callers interact exclusively via opaque pointer handles:

```c
typedef struct SilvestreImageHandle SilvestreImageHandle;
```

### 2.3 Core Exported Functions

```rust
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_load(
    path: *const c_char,
) -> *mut SilvestreImageHandle;

#[no_mangle]
pub unsafe extern "C" fn silvestre_image_from_buffer(
    data: *const u8,
    len: usize,
    width: u32,
    height: u32,
    color_space: u8,
) -> *mut SilvestreImageHandle;

#[no_mangle]
pub unsafe extern "C" fn silvestre_image_save(
    handle: *const SilvestreImageHandle,
    path: *const c_char,
    format: *const c_char,
) -> i32;

#[no_mangle]
pub unsafe extern "C" fn silvestre_image_free(
    handle: *mut SilvestreImageHandle,
);

#[no_mangle]
pub unsafe extern "C" fn silvestre_last_error() -> *const c_char;
```

### 2.4 Error Handling Model
- Functions returning status codes return `0` on success and negative integer error codes on failure (`-1: NullPointer`, `-2: IoError`, `-3: FilterError`, `-4: InvalidParameter`).
- Detailed human-readable error messages are written to a thread-local static buffer, retrieved by `silvestre_last_error()`.

---

## 3. Automated Header Generation

- `cbindgen` generates standard C/C++ header files (`silvestre.h`) during the build process, configured via `cbindgen.toml`.

---

## 4. Verification

- C ABI null pointer safety tests.
- Buffer boundary and memory leak verification across load/apply/free lifecycles.
- Verification of generated `silvestre.h` syntax and compatibility with standard C99 compilers.
