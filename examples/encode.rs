//! Encode an image to WebP with default settings.
//!
//! Opens the bundled JPEG, encodes it to WebP bytes with [`webp::encode`], and writes the result to a file in the
//! `assets/` directory.
//!
//! Run with:
//!
//! ```text
//! cargo run --example encode
//! ```

use std::error::Error;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    // Load any format the `image` crate understands.
    let img = image::open(SOURCE)?;

    // Encode to WebP with sensible defaults (quality 75, compression 4).
    let bytes = webp::encode(&img)?;

    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/encode-example.webp");
    std::fs::write(out, &bytes)?;

    println!("encoded {} bytes -> {}", bytes.len(), out);
    Ok(())
}
