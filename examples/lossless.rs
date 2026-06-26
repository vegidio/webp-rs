//! Encode losslessly using [`webp::EncoderConfig`] (or the `with_lossless` builder).
//!
//! Lossless WebP reconstructs the source pixels exactly — at the cost of a larger file. It's set either by
//! constructing an [`webp::EncoderConfig`] with `lossless: true` or via the `WebpEncoder::with_lossless` builder.
//! Here we encode the same image both lossy and lossless and compare the sizes.
//!
//! Run with:
//!
//! ```text
//! cargo run --example lossless
//! ```

use std::error::Error;

use webp::{EncoderConfig, WebpEncoder};

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open(SOURCE)?;

    // Lossy (default) for comparison.
    let lossy = webp::encode(&img)?;

    // Lossless via an explicit config.
    let config = EncoderConfig {
        lossless: true,
        ..Default::default()
    };
    let mut lossless = Vec::new();
    img.write_with_encoder(WebpEncoder::new_with_config(&mut lossless, config))?;

    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/lossless-example.webp");
    std::fs::write(out, &lossless)?;

    println!("lossy:    {} bytes", lossy.len());
    println!("lossless: {} bytes -> {}", lossless.len(), out);
    Ok(())
}
