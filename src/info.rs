//! Image metadata types returned by probing and decoding.

use image::ColorType;

/// Metadata describing a WebP image, without (necessarily) decoding its pixels.
///
/// WebP is always 8 bits per channel, so there is no bit-depth field — unlike formats
/// such as AVIF, the color type alone fully describes the sample layout.
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    /// Reuse `image`'s enum — no point rolling our own.
    pub color_type: ColorType,
}
