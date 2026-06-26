//! Error-path integration tests: malformed input and buffer mismatches must return
//! `Err` rather than panic.

mod common;

use common::WEBP;

use image::{ExtendedColorType, ImageEncoder};
use webp::WebpEncoder;

#[test]
fn encode_rejects_wrong_length_buffer() {
    // A 1x1 RGBA image needs 4 bytes; hand it 3 so the length check fires.
    let mut out = Vec::new();
    let encoder = WebpEncoder::new(&mut out);
    let result = encoder.write_image(&[0, 0, 0], 1, 1, ExtendedColorType::Rgba8);
    assert!(result.is_err(), "wrong-length buffer should be rejected");
}

#[test]
fn encode_rejects_zero_dimensions() {
    let mut out = Vec::new();
    let encoder = WebpEncoder::new(&mut out);
    let result = encoder.write_image(&[], 0, 0, ExtendedColorType::Rgb8);
    assert!(result.is_err(), "zero dimensions should be rejected");
}

#[test]
fn encode_rejects_unsupported_color_type() {
    // libwebp is 8-bit only; 16-bit input must be rejected as unsupported.
    let mut out = Vec::new();
    let encoder = WebpEncoder::new(&mut out);
    let result = encoder.write_image(&[0; 6], 1, 1, ExtendedColorType::Rgb16);
    assert!(result.is_err(), "16-bit input should be rejected");
}

#[test]
fn decode_rejects_empty_input() {
    assert!(webp::decode(&[]).is_err(), "empty input should not decode");
}

#[test]
fn decode_rejects_garbage_input() {
    assert!(
        webp::decode(b"this is definitely not a webp file").is_err(),
        "non-WebP bytes should not decode"
    );
}

#[test]
fn probe_rejects_truncated_webp() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    // WebP stores its dimensions in the first ~30 bytes, so probe only fails once the
    // header itself is incomplete — truncate hard, past the RIFF header into the frame tag.
    let truncated = &bytes[..16];
    assert!(webp::probe(truncated).is_err(), "truncated WebP should fail to probe");
}
