//! Shared helpers for the integration tests. Each test file is its own crate, so they
//! pull this in with `mod common;`. Not every file uses every helper.
#![allow(dead_code)]

use image::DynamicImage;

pub const JPG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");
pub const WEBP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.webp");

/// Loads the sample JPEG used as an encode source.
pub fn source_image() -> DynamicImage {
    image::open(JPG).expect("load assets/image.jpg")
}

/// True when `bytes` is a RIFF container tagged `WEBP`.
pub fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}
