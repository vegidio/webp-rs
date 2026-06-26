//! Encode with custom settings via the [`webp::WebpEncoder`] builder.
//!
//! Instead of the one-line [`webp::encode`] facade, use `WebpEncoder` directly (through `image`'s `ImageEncoder`
//! trait) to tune quality, compression effort, and threading. Builder methods: `with_quality` / `with_quality_alpha`
//! (0–100, higher = better), `with_compression` (0–6, higher = more effort/smaller file), `with_threads`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example custom_encoder
//! ```

use std::error::Error;

use webp::WebpEncoder;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open(SOURCE)?;

    let mut bytes = Vec::new();
    img.write_with_encoder(
        WebpEncoder::new(&mut bytes)
            .with_quality(80) // 0–100, higher = better quality
            .with_compression(6) // 0–6, higher = more effort, smaller file
            .with_threads(true), // enable multithreaded encoding
    )?;

    let out = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/custom-encoder-example.webp");
    std::fs::write(out, &bytes)?;

    println!("encoded {} bytes (quality 80, compression 6) -> {}", bytes.len(), out);
    Ok(())
}
