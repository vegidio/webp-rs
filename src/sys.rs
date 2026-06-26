//! Raw, unsafe FFI bindings to libwebp, generated at build time by `bindgen` from the
//! `encode.h` / `decode.h` headers bundled with the downloaded static binaries. See `build.rs`.
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
