//! WebP encoder, mirroring the `image` crate's per-format encoder convention.
//!
//! [`WebpEncoder`] is generic over a [`Write`] sink and implements [`ImageEncoder`],
//! so it slots into `DynamicImage::write_with_encoder` exactly like the codecs that
//! ship with the `image` crate (e.g. `JpegEncoder`, `PngEncoder`). Encoding uses
//! libwebp under the hood.

use std::io::Write;
use std::os::raw::c_void;

use image::error::{EncodingError, ImageFormatHint, UnsupportedError, UnsupportedErrorKind};
use image::{ExtendedColorType, ImageEncoder, ImageError, ImageResult};

use crate::error::WebpError;
use crate::ffi;
use crate::sys;

/// Tunable parameters for the libwebp encoder.
///
/// Field ranges and defaults mirror libwebp / the `cwebp` CLI.
pub struct EncoderConfig {
    /// Visual-quality target, 0–100 (higher = better-looking image, larger file);
    /// default 75. Controls how lossy the compression is. Ignored when `lossless` is set.
    /// Maps to `WebPConfig.quality`.
    pub quality: u8,
    /// Quality of the alpha channel, 0–100 (higher = better); default 100.
    /// Maps to `WebPConfig.alpha_quality`.
    pub quality_alpha: u8,
    /// Compression effort / encode-speed trade-off, 0–6 (higher = more effort = slower
    /// encode, smaller file); default 4. This does NOT change the visual-quality target —
    /// that's [`quality`](Self::quality). Maps to `WebPConfig.method`.
    pub compression: u8,
    /// Enable lossless WebP encoding; default `false`. When `true`, [`quality`](Self::quality)
    /// is ignored. Maps to `WebPConfig.lossless`.
    pub lossless: bool,
    /// Enable multi-threaded encoding (libwebp `thread_level`, on/off only); default
    /// `false`, matching libwebp/`cwebp`. libwebp's intra-image parallelism is limited (it
    /// mainly overlaps the alpha-plane encode with the main bitstream), so the speedup is
    /// modest. Prefer leaving this off when batch-encoding many images concurrently (one
    /// image per thread) to avoid CPU oversubscription; enable it when encoding a single
    /// large image in isolation.
    pub threads: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            quality: 75,
            quality_alpha: 100,
            compression: 4,
            lossless: false,
            threads: false,
        }
    }
}

/// WebP encoder writing to `W`, using libwebp.
///
/// # Example
/// ```no_run
/// use webp::WebpEncoder;
/// use image::ImageEncoder;
///
/// let img = image::open("photo.png")?;
/// let mut buf = Vec::new();
/// img.write_with_encoder(WebpEncoder::new(&mut buf))?;
/// # Ok::<(), image::ImageError>(())
/// ```
pub struct WebpEncoder<W: Write> {
    writer: W,
    config: EncoderConfig,
}

impl<W: Write> WebpEncoder<W> {
    /// Create an encoder writing to `w` with default settings.
    pub fn new(w: W) -> Self {
        Self {
            writer: w,
            config: EncoderConfig::default(),
        }
    }

    /// Create an encoder writing to `w` with an explicit configuration.
    pub fn new_with_config(w: W, config: EncoderConfig) -> Self {
        Self { writer: w, config }
    }

    /// Visual-quality target, 0–100 (higher = better-looking image, larger file).
    /// Ignored when lossless encoding is enabled.
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.config.quality = quality;
        self
    }

    /// Quality of the alpha channel, 0–100 (higher = better).
    pub fn with_quality_alpha(mut self, quality_alpha: u8) -> Self {
        self.config.quality_alpha = quality_alpha;
        self
    }

    /// Compression effort / encode-speed trade-off, 0–6 (higher = more effort = slower
    /// encode, smaller file). Does NOT change the visual-quality target — see
    /// [`with_quality`](Self::with_quality).
    pub fn with_compression(mut self, compression: u8) -> Self {
        self.config.compression = compression;
        self
    }

    /// Enable lossless WebP encoding. When enabled, the quality setting is ignored.
    pub fn with_lossless(mut self, lossless: bool) -> Self {
        self.config.lossless = lossless;
        self
    }

    /// Enable multi-threaded encoding (on/off only). See [`EncoderConfig::threads`] for
    /// guidance on when this actually helps.
    pub fn with_threads(mut self, threads: bool) -> Self {
        self.config.threads = threads;
        self
    }
}

/// How an input pixel buffer maps onto the RGB(A) image libwebp consumes. WebP is 8-bit
/// only, so there is no sample-size dimension here (every sample is one byte).
struct Layout {
    /// Channels per pixel in the *input* `buf` (1=L, 2=La, 3=Rgb, 4=Rgba).
    src_channels: usize,
    /// Channels in the buffer handed to libwebp (3=RGB, 4=RGBA).
    rgb_channels: usize,
    /// Whether the input is grayscale and must be expanded to RGB/RGBA.
    gray: bool,
    /// Whether the input carries an alpha channel.
    alpha: bool,
}

/// Maps a supported [`ExtendedColorType`] to its [`Layout`], or `None` if unsupported.
/// libwebp is 8-bit only, so 16-bit inputs are rejected as unsupported.
fn layout_for(color_type: ExtendedColorType) -> Option<Layout> {
    use ExtendedColorType as E;

    let l = |src, rgb_ch, gray, alpha| {
        Some(Layout {
            src_channels: src,
            rgb_channels: rgb_ch,
            gray,
            alpha,
        })
    };

    match color_type {
        E::L8 => l(1, 3, true, false),
        E::La8 => l(2, 4, true, true),
        E::Rgb8 => l(3, 3, false, false),
        E::Rgba8 => l(4, 4, false, true),
        _ => None,
    }
}

/// Expands a grayscale buffer (L or La) into RGB/RGBA by replicating luma into the three
/// color channels (8-bit samples).
fn expand_gray(buf: &[u8], alpha: bool) -> Vec<u8> {
    let in_ch = if alpha { 2 } else { 1 };
    let out_ch = if alpha { 4 } else { 3 };
    let pixels = buf.len() / in_ch;
    let mut out = Vec::with_capacity(pixels * out_ch);

    for i in 0..pixels {
        let base = i * in_ch;
        let luma = buf[base];
        out.push(luma);
        out.push(luma);
        out.push(luma);

        if alpha {
            out.push(buf[base + 1]);
        }
    }

    out
}

impl EncoderConfig {
    /// Runs the full libwebp encode pipeline, returning the encoded WebP bytes.
    fn encode(
        &self,
        buf: &[u8],
        width: u32,
        height: u32,
        color_type: ExtendedColorType,
    ) -> Result<Vec<u8>, EncodeError> {
        if width == 0 || height == 0 {
            return Err(EncodeError::Webp(WebpError::InvalidDimensions { width, height }));
        }

        let layout = layout_for(color_type).ok_or(EncodeError::Unsupported(color_type))?;

        let expected = width as usize * height as usize * layout.src_channels;
        if buf.len() != expected {
            return Err(EncodeError::Webp(WebpError::Encode(format!(
                "buffer length {} does not match {width}x{height} with {} channels",
                buf.len(),
                layout.src_channels,
            ))));
        }

        // Either borrow the caller's buffer directly or build an expanded grayscale copy.
        let expanded;
        let pixels: &[u8] = if layout.gray {
            expanded = expand_gray(buf, layout.alpha);
            &expanded
        } else {
            buf
        };

        // SAFETY: the config and picture are stack-allocated and initialized via the
        // version-checked `*Internal` entry points; the picture and memory writer are
        // freed on every path below.
        unsafe { self.encode_raw(pixels, width, height, &layout) }
    }

    /// Inner half of [`encode`](Self::encode): runs libwebp against an already-validated
    /// RGB/RGBA pixel buffer.
    ///
    /// # Safety
    /// `pixels` must contain exactly `width * height * layout.rgb_channels` bytes.
    unsafe fn encode_raw(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        layout: &Layout,
    ) -> Result<Vec<u8>, EncodeError> {
        unsafe {
            let mut config: sys::WebPConfig = std::mem::zeroed();
            if sys::WebPConfigInitInternal(
                &mut config,
                sys::WebPPreset_WEBP_PRESET_DEFAULT,
                self.quality as f32,
                sys::WEBP_ENCODER_ABI_VERSION as i32,
            ) == 0
            {
                return Err(EncodeError::Webp(WebpError::EncoderInit(
                    "WebPConfigInit failed (version mismatch)".into(),
                )));
            }

            config.quality = self.quality as f32;
            config.alpha_quality = self.quality_alpha as i32;
            config.method = self.compression as i32;
            config.lossless = self.lossless as i32;
            config.thread_level = self.threads as i32;

            if sys::WebPValidateConfig(&config) == 0 {
                return Err(EncodeError::Webp(WebpError::EncoderInit(
                    "invalid encoder configuration".into(),
                )));
            }

            let mut picture: sys::WebPPicture = std::mem::zeroed();
            if sys::WebPPictureInitInternal(&mut picture, sys::WEBP_ENCODER_ABI_VERSION as i32) == 0 {
                return Err(EncodeError::Webp(WebpError::EncoderInit(
                    "WebPPictureInit failed (version mismatch)".into(),
                )));
            }

            picture.use_argb = 1;
            picture.width = width as i32;
            picture.height = height as i32;

            let stride = (width as usize * layout.rgb_channels) as i32;
            let imported = if layout.alpha {
                sys::WebPPictureImportRGBA(&mut picture, pixels.as_ptr(), stride)
            } else {
                sys::WebPPictureImportRGB(&mut picture, pixels.as_ptr(), stride)
            };
            if imported == 0 {
                sys::WebPPictureFree(&mut picture);
                return Err(EncodeError::Webp(WebpError::Encode(
                    "failed to import pixels (out of memory?)".into(),
                )));
            }

            let mut writer: sys::WebPMemoryWriter = std::mem::zeroed();
            sys::WebPMemoryWriterInit(&mut writer);
            picture.writer = Some(sys::WebPMemoryWrite);
            picture.custom_ptr = &mut writer as *mut _ as *mut c_void;

            let ok = sys::WebPEncode(&config, &mut picture);
            let encoded = if ok != 0 {
                if writer.mem.is_null() || writer.size == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(std::slice::from_raw_parts(writer.mem, writer.size).to_vec())
                }
            } else {
                Err(EncodeError::Webp(WebpError::Encode(ffi::encode_error_message(
                    picture.error_code,
                ))))
            };

            sys::WebPMemoryWriterClear(&mut writer);
            sys::WebPPictureFree(&mut picture);
            encoded
        }
    }
}

/// Internal encode failure, distinguishing unsupported inputs (which become
/// `ImageError::Unsupported`) from libwebp/runtime failures.
enum EncodeError {
    Unsupported(ExtendedColorType),
    Webp(WebpError),
}

impl From<EncodeError> for ImageError {
    fn from(err: EncodeError) -> Self {
        match err {
            EncodeError::Unsupported(color_type) => ImageError::Unsupported(UnsupportedError::from_format_and_kind(
                ImageFormatHint::Name("WebP".into()),
                UnsupportedErrorKind::Color(color_type),
            )),
            EncodeError::Webp(e) => ImageError::Encoding(EncodingError::new(ImageFormatHint::Name("WebP".into()), e)),
        }
    }
}

impl<W: Write> ImageEncoder for WebpEncoder<W> {
    fn write_image(mut self, buf: &[u8], width: u32, height: u32, color_type: ExtendedColorType) -> ImageResult<()> {
        let encoded = self.config.encode(buf, width, height, color_type)?;
        self.writer.write_all(&encoded).map_err(ImageError::IoError)?;
        Ok(())
    }
}
