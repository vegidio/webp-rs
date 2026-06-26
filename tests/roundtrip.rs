//! Encode→decode round-trip tests across pixel layouts.

mod common;

use common::source_image;
use image::{DynamicImage, GenericImageView};

#[test]
fn roundtrip_preserves_dimensions() {
    let img = source_image();
    let (w, h) = (img.width(), img.height());

    let bytes = webp::encode(&img).expect("encode");
    let decoded = webp::decode(&bytes).expect("decode");

    assert_eq!(decoded.dimensions(), (w, h));
}

/// Encode an image with explicit alpha so the RGBA path is exercised.
#[test]
fn roundtrip_rgba() {
    let img = DynamicImage::ImageRgba8(source_image().to_rgba8());
    let bytes = webp::encode(&img).expect("encode rgba");
    let decoded = webp::decode(&bytes).expect("decode rgba");
    assert_eq!(decoded.dimensions(), img.dimensions());
}

/// Grayscale input must be expanded to RGB (libwebp consumes RGB/RGBA only).
#[test]
fn roundtrip_grayscale() {
    let img = DynamicImage::ImageLuma8(source_image().to_luma8());
    let bytes = webp::encode(&img).expect("encode grayscale");
    let decoded = webp::decode(&bytes).expect("decode grayscale");
    assert_eq!(decoded.dimensions(), img.dimensions());
}
