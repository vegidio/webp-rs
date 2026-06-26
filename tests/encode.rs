//! Encoder-focused integration tests.

mod common;

use common::{is_webp, source_image};

#[test]
fn encode_produces_valid_webp() {
    let img = source_image();
    let bytes = webp::encode(&img).expect("encode");
    assert!(!bytes.is_empty(), "encoded output should not be empty");
    assert!(is_webp(&bytes), "output should be a valid WebP stream");
}
