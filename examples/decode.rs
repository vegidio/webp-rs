//! Decode a WebP file into an image and save it in another format.
//!
//! Reads the bundled WebP, decodes it to a [`image::DynamicImage`] with [`webp::decode`], and saves it as a PNG in the
//! `assets/` directory.
//!
//! Run with:
//!
//! ```text
//! cargo run --example decode
//! ```

use std::error::Error;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.webp");

fn main() -> Result<(), Box<dyn Error>> {
    let bytes = std::fs::read(SOURCE)?;

    // Decode WebP bytes into a DynamicImage.
    let img = webp::decode(&bytes)?;

    // `image` infers the output format from the file extension.
    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/decode-example.png");
    img.save(out)?;

    println!("decoded to PNG -> {}", out);
    Ok(())
}
