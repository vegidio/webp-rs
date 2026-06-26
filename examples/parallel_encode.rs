//! Encode several images concurrently.
//!
//! libwebp is thread-safe for independent encodes, so spawning several encodes at once — each with its own
//! configuration — is perfectly safe. The efficient pattern for batch work is one image per thread, with libwebp's
//! own intra-image threading left off (the default) to avoid CPU oversubscription.
//!
//! Run with:
//!
//! ```text
//! cargo run --example parallel_encode
//! ```

use std::error::Error;
use std::thread;

const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/image.jpg");

fn main() -> Result<(), Box<dyn Error>> {
    let img = image::open(SOURCE)?;

    // Spawn several encodes at once. Unlike some AV1 encoders, libwebp has no shared global state, so the threads can
    // even use different settings safely.
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let img = img.clone();
            thread::spawn(move || (i, webp::encode(&img).expect("encode")))
        })
        .collect();

    for handle in handles {
        let (i, bytes) = handle.join().expect("thread panicked");
        println!("thread {i}: encoded {} bytes", bytes.len());
    }

    Ok(())
}
