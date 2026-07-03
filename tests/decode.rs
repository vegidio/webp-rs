//! Decoder- and probe-focused integration tests against the bundled WebP asset.

mod common;

use common::WEBP;

use std::io::Cursor;

use image::{ColorType, DynamicImage, ImageDecoder};
use webp::{WebpDecoder, WebpEncoder};

#[test]
fn probe_reads_header_only() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    let info = webp::probe(&bytes).expect("probe");

    assert!(info.width > 0 && info.height > 0);
}

#[test]
fn decodes_bundled_webp_asset() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    let img = webp::decode(&bytes).expect("decode bundled asset");
    assert!(img.width() > 0 && img.height() > 0);
}

/// A WebP carrying a real (non-opaque) alpha channel must report `Rgba8` and be decoded
/// through libwebp's RGBA colorspace path, preserving the alpha bytes.
#[test]
fn decodes_alpha_as_rgba8() {
    // Synthetic RGBA image with a genuinely transparent checkerboard, so the encoded WebP
    // keeps its alpha plane instead of libwebp dropping a fully-opaque one.
    let mut rgba = image::RgbaImage::new(8, 8);
    for (x, y, px) in rgba.enumerate_pixels_mut() {
        let a = if (x + y) % 2 == 0 { 0 } else { 255 };
        *px = image::Rgba([(x * 32) as u8, (y * 32) as u8, 128, a]);
    }
    let original = DynamicImage::ImageRgba8(rgba);

    let mut buf = Vec::new();
    original
        .write_with_encoder(WebpEncoder::new(&mut buf).with_lossless(true))
        .expect("encode lossless rgba");

    // Header-level: the decoder must classify this as RGBA before any pixels are decoded.
    let decoder = WebpDecoder::new(Cursor::new(&buf)).expect("construct decoder");
    assert_eq!(decoder.color_type(), ColorType::Rgba8, "alpha input should decode as Rgba8");

    // Pixel-level: lossless must reconstruct the alpha plane exactly. (Note: libwebp zeroes
    // the RGB under fully-transparent pixels, so we compare the alpha channel, not full pixels.)
    let decoded = webp::decode(&buf).expect("decode rgba").to_rgba8();
    let expected = original.to_rgba8();
    assert_eq!(decoded.dimensions(), expected.dimensions());
    for (got, want) in decoded.pixels().zip(expected.pixels()) {
        assert_eq!(got[3], want[3], "alpha channel must round-trip exactly under lossless");
    }
}

/// `read_image` must reject an output buffer whose length doesn't match the frame.
#[test]
fn read_image_rejects_wrong_buffer_length() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    let decoder = WebpDecoder::new(Cursor::new(&bytes)).expect("construct decoder");

    let mut too_small = vec![0u8; 4];
    assert!(
        decoder.read_image(&mut too_small).is_err(),
        "an undersized output buffer must be rejected"
    );
}

/// The `with_threads` builder toggle must still decode correctly.
#[test]
fn decodes_with_threads_enabled() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    let decoder = WebpDecoder::new(Cursor::new(&bytes))
        .expect("construct decoder")
        .with_threads(true);

    let img = DynamicImage::from_decoder(decoder).expect("multi-threaded decode");
    assert!(img.width() > 0 && img.height() > 0);
}
