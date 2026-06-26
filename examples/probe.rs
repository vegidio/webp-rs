//! Read a WebP header without decoding its pixels using [`webp::probe`].
//!
//! Returns a [`webp::ImageInfo`] with dimensions and color type — useful for validation or thumbnailing pipelines
//! where you don't need the decoded image.
//!
//! Run with:
//!
//! ```text
//! cargo run --example probe
//! ```

use std::error::Error;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.webp");

fn main() -> Result<(), Box<dyn Error>> {
    let bytes = std::fs::read(SOURCE)?;

    let info = webp::probe(&bytes)?;

    println!("dimensions: {}x{}", info.width, info.height);
    println!("color type: {:?}", info.color_type);
    Ok(())
}
