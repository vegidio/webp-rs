//! Encoder-focused integration tests.

mod common;

use common::{is_webp, source_image};

use image::{DynamicImage, GenericImageView};
use webp::{EncoderConfig, WebpEncoder};

#[test]
fn encode_produces_valid_webp() {
    let img = source_image();
    let bytes = webp::encode(&img).expect("encode");
    assert!(!bytes.is_empty(), "encoded output should not be empty");
    assert!(is_webp(&bytes), "output should be a valid WebP stream");
}

/// The typed `encode_buffer` facade must encode an `ImageBuffer` without going through
/// `DynamicImage`.
#[test]
fn encode_buffer_encodes_typed_image() {
    let rgba = source_image().to_rgba8();
    let (w, h) = rgba.dimensions();

    let bytes = webp::encode_buffer(&rgba).expect("encode_buffer");
    assert!(is_webp(&bytes), "encode_buffer output should be a valid WebP stream");
    assert_eq!(webp::decode(&bytes).expect("decode").dimensions(), (w, h));
}

/// `new_with_config` must apply a fully-specified `EncoderConfig`.
#[test]
fn encodes_with_explicit_config() {
    let img = source_image();
    let (w, h) = img.dimensions();

    let config = EncoderConfig {
        quality: 40,
        quality_alpha: 60,
        compression: 2,
        lossless: false,
        threads: false,
    };

    let mut buf = Vec::new();
    img.write_with_encoder(WebpEncoder::new_with_config(&mut buf, config))
        .expect("encode with explicit config");
    assert!(is_webp(&buf), "explicit-config output should be a valid WebP stream");
    assert_eq!(webp::decode(&buf).expect("decode").dimensions(), (w, h));
}

/// Grayscale-with-alpha (`La8`) input exercises the `layout_for` LumaA arm and the
/// alpha branch of `expand_gray`.
#[test]
fn roundtrip_grayscale_alpha() {
    let img = DynamicImage::ImageLumaA8(source_image().to_luma_alpha8());
    let bytes = webp::encode(&img).expect("encode luma+alpha");
    assert!(is_webp(&bytes), "luma+alpha output should be a valid WebP stream");

    let decoded = webp::decode(&bytes).expect("decode luma+alpha");
    assert_eq!(decoded.dimensions(), img.dimensions());
}
