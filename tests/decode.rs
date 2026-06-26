//! Decoder- and probe-focused integration tests against the bundled WebP asset.

mod common;

use common::WEBP;

#[test]
fn probe_reads_header_only() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    let info = webp::probe(&bytes).expect("probe");

    assert!(info.width > 0 && info.height > 0);
}

#[test]
fn decodes_bundled_webp_asset() {
    let bytes = std::fs::read(WEBP).expect("read assets/image.webp");
    let img = webp::decode(&bytes).expect("decode bundled asset");
    assert!(img.width() > 0 && img.height() > 0);
}
