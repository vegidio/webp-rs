//! WebP decoder, mirroring the `image` crate's per-format decoder convention.
//!
//! [`WebpDecoder`] is generic over a [`Read`] source and implements [`ImageDecoder`],
//! so it slots into `DynamicImage::from_decoder` exactly like the codecs that ship with
//! the `image` crate (e.g. `JpegDecoder`, `PngDecoder`). Decoding uses libwebp under the
//! hood.

use std::io::Read;

use image::error::{DecodingError, ImageFormatHint};
use image::{ColorType, ImageDecoder, ImageError, ImageResult};

use crate::error::WebpError;
use crate::ffi;
use crate::sys;

/// Tunable parameters for the libwebp decoder.
#[derive(Default)]
pub struct DecoderConfig {
    /// Enable multi-threaded decoding (libwebp `use_threads`, on/off only); default
    /// `false`. The speedup is modest; leave off when decoding many images concurrently.
    pub threads: bool,
}

/// WebP decoder reading from `R`, using libwebp.
///
/// The bitstream header is parsed eagerly in [`new`](WebpDecoder::new) so that
/// [`dimensions`](ImageDecoder::dimensions) and [`color_type`](ImageDecoder::color_type)
/// are available before the pixels are decoded.
///
/// # Example
/// ```no_run
/// use webp::WebpDecoder;
/// use image::DynamicImage;
/// use std::io::Cursor;
///
/// # let bytes: Vec<u8> = Vec::new();
/// let decoder = WebpDecoder::new(Cursor::new(&bytes))?;
/// let img = DynamicImage::from_decoder(decoder)?;
/// # Ok::<(), image::ImageError>(())
/// ```
pub struct WebpDecoder<R: Read> {
    /// Owned compressed bytes; libwebp re-reads these when the frame is decoded.
    data: Vec<u8>,
    config: DecoderConfig,
    width: u32,
    height: u32,
    alpha_present: bool,
    /// Marker to keep the `R` type parameter; the reader is fully drained in `new`.
    _reader: std::marker::PhantomData<R>,
}

impl<R: Read> WebpDecoder<R> {
    /// Create a decoder from `r`, reading the bitstream header eagerly so that
    /// [`dimensions`](ImageDecoder::dimensions) and [`color_type`](ImageDecoder::color_type)
    /// are available before the frame is decoded.
    pub fn new(mut r: R) -> ImageResult<Self> {
        let mut data = Vec::new();
        r.read_to_end(&mut data).map_err(ImageError::IoError)?;

        // SAFETY: `features` is fully initialized by `WebPGetFeaturesInternal`; `data`
        // outlives this call.
        unsafe {
            let mut features: sys::WebPBitstreamFeatures = std::mem::zeroed();
            let status = sys::WebPGetFeaturesInternal(
                data.as_ptr(),
                data.len(),
                &mut features,
                sys::WEBP_DECODER_ABI_VERSION as i32,
            );
            if !ffi::is_ok(status) {
                return Err(to_image_error(WebpError::DecoderInit(ffi::status_message(status))));
            }

            Ok(Self {
                width: features.width as u32,
                height: features.height as u32,
                alpha_present: features.has_alpha != 0,
                data,
                config: DecoderConfig::default(),
                _reader: std::marker::PhantomData,
            })
        }
    }

    /// Enable multi-threaded decoding (on/off only).
    pub fn with_threads(mut self, threads: bool) -> Self {
        self.config.threads = threads;
        self
    }

    /// Channels in the decoded RGB output (3 without alpha, 4 with).
    fn channels(&self) -> usize {
        if self.alpha_present { 4 } else { 3 }
    }
}

impl<R: Read> ImageDecoder for WebpDecoder<R> {
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn color_type(&self) -> ColorType {
        if self.alpha_present {
            ColorType::Rgba8
        } else {
            ColorType::Rgb8
        }
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()> {
        let expected = self.width as usize * self.height as usize * self.channels();
        if buf.len() != expected {
            return Err(to_image_error(WebpError::Decode(format!(
                "output buffer length {} does not match expected {expected}",
                buf.len()
            ))));
        }

        // SAFETY: the decoder config is initialized via the version-checked entry point;
        // the output buffer is caller-owned (`is_external_memory = 1`) and sized to match.
        unsafe {
            let mut config: sys::WebPDecoderConfig = std::mem::zeroed();
            if sys::WebPInitDecoderConfigInternal(&mut config, sys::WEBP_DECODER_ABI_VERSION as i32) == 0 {
                return Err(to_image_error(WebpError::DecoderInit(
                    "WebPInitDecoderConfig failed (version mismatch)".into(),
                )));
            }

            config.options.use_threads = self.config.threads as i32;
            config.output.colorspace = if self.alpha_present {
                sys::WEBP_CSP_MODE_MODE_RGBA
            } else {
                sys::WEBP_CSP_MODE_MODE_RGB
            };
            config.output.is_external_memory = 1;
            config.output.u.RGBA.rgba = buf.as_mut_ptr();
            config.output.u.RGBA.stride = (self.width as usize * self.channels()) as i32;
            config.output.u.RGBA.size = buf.len();

            let status = sys::WebPDecode(self.data.as_ptr(), self.data.len(), &mut config);
            // Releases any decoder-internal scratch; the external output buffer is untouched.
            sys::WebPFreeDecBuffer(&mut config.output);

            if !ffi::is_ok(status) {
                return Err(to_image_error(WebpError::Decode(ffi::status_message(status))));
            }
        }
        Ok(())
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        self.read_image(buf)
    }
}

/// Wraps a [`WebpError`] as an `image` decoding error.
fn to_image_error(err: WebpError) -> ImageError {
    ImageError::Decoding(DecodingError::new(ImageFormatHint::Name("WebP".into()), err))
}
