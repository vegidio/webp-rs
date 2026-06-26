//! `webp-rs` — encode and decode WebP images via libwebp.
//!
//! The libwebp C library (plus its `sharpyuv` support dependency) is downloaded as a prebuilt **static** library at
//! build time and linked directly into this crate, so consumers do not need libwebp installed on the host. See
//! `build.rs`.
//!
//! The API mirrors the `image` crate's codec conventions:
//! * [`WebpEncoder`] / [`WebpDecoder`] implement `image`'s `ImageEncoder` / `ImageDecoder` traits, so they plug into
//!   `DynamicImage::write_with_encoder` / `DynamicImage::from_decoder` just like the codecs bundled with `image`.
//! * a thin facade ([`encode`], [`encode_buffer`], [`decode`], [`probe`]) wraps those for one-line convenience.
//!
//! WebP is an 8-bit format: inputs are encoded as 8-bit RGB/RGBA and decoded back to the same.

mod decoder;
mod encoder;
mod error;
mod ffi;
mod info;
mod sys;

pub use decoder::{DecoderConfig, WebpDecoder};
pub use encoder::{EncoderConfig, WebpEncoder};
pub use error::{Result, WebpError};
pub use info::ImageInfo;

use std::io::Cursor;
use std::ops::Deref;

use image::{DynamicImage, EncodableLayout, ImageBuffer, ImageDecoder, PixelWithColorType};

/// Returns the version string of the linked libwebp encoder, e.g. `"1.5.0"`.
///
/// libwebp exposes its version as a packed integer `(major << 16) | (minor << 8) | patch`,
/// which this function decodes into a dotted string.
pub fn libwebp_version() -> String {
    // SAFETY: `WebPGetEncoderVersion` takes no arguments and returns a plain integer.
    let v = unsafe { sys::WebPGetEncoderVersion() } as u32;
    let major = (v >> 16) & 0xff;
    let minor = (v >> 8) & 0xff;
    let patch = v & 0xff;
    format!("{major}.{minor}.{patch}")
}

/// Encode a [`DynamicImage`] to WebP bytes using sensible defaults.
///
/// # Example
/// ```no_run
/// let img = image::open("photo.png")?;
/// let webp_bytes = webp::encode(&img)?;
/// # Ok::<(), webp::WebpError>(())
/// ```
pub fn encode(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    image.write_with_encoder(WebpEncoder::new(&mut buf))?;
    Ok(buf)
}

/// Encode a typed [`ImageBuffer`] directly, avoiding the runtime dispatch
/// overhead of [`DynamicImage`]. Prefer this when you already know your
/// pixel type at compile time.
///
/// # Example
/// ```no_run
/// use image::RgbaImage;
/// let img: RgbaImage = image::open("photo.png")?.into_rgba8();
/// let webp_bytes = webp::encode_buffer(&img)?;
/// # Ok::<(), webp::WebpError>(())
/// ```
pub fn encode_buffer<P, C>(buffer: &ImageBuffer<P, C>) -> Result<Vec<u8>>
where
    P: PixelWithColorType,
    [P::Subpixel]: EncodableLayout,
    C: Deref<Target = [P::Subpixel]>,
{
    let mut buf = Vec::new();
    buffer.write_with_encoder(WebpEncoder::new(&mut buf))?;
    Ok(buf)
}

/// Decode WebP bytes into a [`DynamicImage`].
///
/// # Example
/// ```no_run
/// # let webp_bytes: Vec<u8> = Vec::new();
/// let img = webp::decode(&webp_bytes)?;
/// img.save("output.png")?;
/// # Ok::<(), webp::WebpError>(())
/// ```
pub fn decode(data: &[u8]) -> Result<DynamicImage> {
    let decoder = WebpDecoder::new(Cursor::new(data))?;
    Ok(DynamicImage::from_decoder(decoder)?)
}

/// Read only the image header — no pixel decode.
/// Useful for validation or thumbnailing pipelines.
///
/// # Example
/// ```no_run
/// # let webp_bytes: Vec<u8> = Vec::new();
/// let info = webp::probe(&webp_bytes)?;
/// println!("{}x{} ({:?})", info.width, info.height, info.color_type);
/// # Ok::<(), webp::WebpError>(())
/// ```
pub fn probe(data: &[u8]) -> Result<ImageInfo> {
    let decoder = WebpDecoder::new(Cursor::new(data))?;
    let (width, height) = decoder.dimensions();
    Ok(ImageInfo {
        width,
        height,
        color_type: decoder.color_type(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: calling into libwebp proves the static binaries are linked and
    /// callable end-to-end.
    #[test]
    fn reports_libwebp_version() {
        let version = libwebp_version();
        println!("linked libwebp version: {version}");
        assert!(!version.is_empty());
    }
}
