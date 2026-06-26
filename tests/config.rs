//! Encoder configuration tests: non-default builder settings must still produce a
//! valid WebP that decodes back to the original dimensions.

mod common;

use common::{is_webp, source_image};

use image::{DynamicImage, GenericImageView};
use webp::WebpEncoder;

#[test]
fn encodes_with_custom_config() {
    let img = source_image();
    let (w, h) = (img.width(), img.height());

    // Custom quality + compression effort.
    let mut buf = Vec::new();
    img.write_with_encoder(WebpEncoder::new(&mut buf).with_quality(20).with_compression(6))
        .expect("encode with custom quality/compression");
    assert!(
        is_webp(&buf),
        "custom quality/compression output should be a valid WebP stream"
    );
    assert_eq!(webp::decode(&buf).expect("decode").dimensions(), (w, h));

    // Multi-threaded encode.
    let mut buf = Vec::new();
    img.write_with_encoder(WebpEncoder::new(&mut buf).with_threads(true))
        .expect("encode multi-threaded");
    assert!(is_webp(&buf), "multi-threaded output should be a valid WebP stream");

    // RGBA with custom alpha quality (exercises the separate alpha-plane encoder).
    let rgba = DynamicImage::ImageRgba8(source_image().to_rgba8());
    let mut buf = Vec::new();
    rgba.write_with_encoder(WebpEncoder::new(&mut buf).with_quality_alpha(30))
        .expect("encode rgba with custom alpha quality");
    assert!(is_webp(&buf), "rgba output should be a valid WebP stream");
    assert_eq!(webp::decode(&buf).expect("decode rgba").dimensions(), (w, h));
}

#[test]
fn lossless_roundtrips_exactly() {
    // A small synthetic RGBA image; lossless must reconstruct it bit-for-bit.
    let mut rgba = image::RgbaImage::new(16, 16);
    for (x, y, px) in rgba.enumerate_pixels_mut() {
        *px = image::Rgba([(x * 16) as u8, (y * 16) as u8, ((x + y) * 8) as u8, 255]);
    }
    let original = DynamicImage::ImageRgba8(rgba);

    let mut buf = Vec::new();
    original
        .write_with_encoder(WebpEncoder::new(&mut buf).with_lossless(true))
        .expect("encode lossless");
    assert!(is_webp(&buf), "lossless output should be a valid WebP stream");

    let decoded = webp::decode(&buf).expect("decode lossless");
    assert_eq!(
        decoded.to_rgba8(),
        original.to_rgba8(),
        "lossless round-trip must be exact"
    );
}
