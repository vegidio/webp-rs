//! Build script for `webp-rs`.
//!
//! At build time this script obtains the prebuilt **bundle** libwebp binaries for the
//! current target, links them statically into the crate, and generates the raw FFI bindings
//! from the bundled `encode.h` / `decode.h` headers.
//!
//! Binaries come from: https://github.com/vegidio/binaries-webp/releases
//!
//! The binaries are normally downloaded from the pinned release and cached under the
//! build's `OUT_DIR`. To build offline (or against a custom build of libwebp), set the
//! `WEBP_BINARIES_DIR` environment variable to a directory that contains `include/` and
//! `lib/` subdirectories laid out like the extracted `bundle_<os>_<arch>.zip` archives.

use bindgen::callbacks::{ItemInfo, ParseCallbacks};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Version of the `binaries-webp` release to download.
const VERSION: &str = "26.10.1";

/// The bundle archive ships a single static library (`libwebp.a`, or `webp.lib` on MSVC)
/// that merges libwebp (encoder + decoder) with `libsharpyuv`.
const STATIC_LIB: &str = "webp";

/// Prefix the bundle adds to every symbol, libwebp's own API included: FFmpeg bundles libwebp
/// too, so the API itself had to be made private to avoid clashing with a static FFmpeg.
const SYMBOL_PREFIX: &str = "webprs_";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=WEBP_BINARIES_DIR");

    let binaries_dir = locate_binaries();
    let lib_dir = binaries_dir.join("lib");
    let include_dir = binaries_dir.join("include");

    emit_link_directives(&lib_dir);
    generate_bindings(&include_dir);
}

/// Returns the directory containing the `include/` and `lib/` subdirectories, either from
/// the `WEBP_BINARIES_DIR` override or by downloading + extracting the pinned release.
fn locate_binaries() -> PathBuf {
    if let Ok(dir) = env::var("WEBP_BINARIES_DIR") {
        let dir = PathBuf::from(dir);
        assert!(
            dir.join("include").is_dir() && dir.join("lib").is_dir(),
            "WEBP_BINARIES_DIR ({}) must contain `include/` and `lib/` subdirectories",
            dir.display()
        );
        return dir;
    }

    download_and_extract()
}

/// Downloads the bundle archive for the current target into `OUT_DIR` and extracts it.
/// Extraction is skipped if a previous build already populated the cache directory.
fn download_and_extract() -> PathBuf {
    let archive = archive_name();
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    // Distinct from the old `binaries-webp-{VERSION}` static cache, so a stale static
    // download is never reused.
    let cache_dir = out_dir.join(format!("binaries-webp-bundle-{VERSION}"));

    // Idempotent: a fully extracted cache from a previous build is reused as-is.
    if cache_dir.join("lib").is_dir() && cache_dir.join("include").is_dir() {
        return cache_dir;
    }

    let url = format!("https://github.com/vegidio/binaries-webp/releases/download/{VERSION}/{archive}");
    eprintln!("webp-rs: downloading {url}");

    let bytes = download(&url);
    extract_zip(&bytes, &cache_dir);
    cache_dir
}

/// Maps the Cargo target triple components to the release archive file name,
/// e.g. `bundle_osx_arm64.zip`. Windows GNU targets use the `mingw` (`.a`) archive, MSVC
/// targets the `windows` (`.lib`) one.
fn archive_name() -> String {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let os = match (target_os.as_str(), target_env.as_str()) {
        ("linux", _) => "linux",
        ("macos", _) => "osx",
        ("windows", "gnu") => "mingw",
        ("windows", _) => "windows",
        (other, _) => panic!("webp-rs: unsupported target OS `{other}`"),
    };

    let arch = match target_arch.as_str() {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => panic!("webp-rs: unsupported target architecture `{other}`"),
    };

    format!("bundle_{os}_{arch}.zip")
}

/// Downloads the given URL into memory, following redirects.
fn download(url: &str) -> Vec<u8> {
    let mut reader = ureq::get(url)
        .call()
        .unwrap_or_else(|e| panic!("webp-rs: failed to download {url}: {e}"))
        .into_body()
        .into_reader();

    let mut bytes = Vec::new();
    io::copy(&mut reader, &mut bytes)
        .unwrap_or_else(|e| panic!("webp-rs: failed to read response body from {url}: {e}"));
    bytes
}

/// Extracts a zip archive (held entirely in memory) into `dest`.
fn extract_zip(bytes: &[u8], dest: &Path) {
    let reader = io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(reader).expect("webp-rs: invalid zip archive");

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).expect("webp-rs: corrupt zip entry");
        let Some(rel_path) = entry.enclosed_name() else {
            continue; // skip unsafe / absolute paths
        };
        let out_path = dest.join(rel_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).unwrap();
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut out_file = fs::File::create(&out_path)
            .unwrap_or_else(|e| panic!("webp-rs: cannot create {}: {e}", out_path.display()));
        io::copy(&mut entry, &mut out_file).expect("webp-rs: failed to extract file");
    }
}

/// Tells Cargo/rustc where the bundled static library lives and links it, plus the system
/// libraries libwebp depends on.
fn emit_link_directives(lib_dir: &Path) {
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static={STATIC_LIB}");

    // libwebp / libsharpyuv are C and only need libm + pthreads on Linux. macOS and mingw get
    // everything from rustc's default libs, and the MSVC `.lib` archive links its C runtime
    // via the objects' default-lib directives.
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=pthread");
    }
}

/// Points every generated `extern` item at the bundle's prefixed symbol (`WebPEncodeRGBA` →
/// `webprs_WebPEncodeRGBA`) while keeping the Rust-side name unchanged.
///
/// bindgen emits an overridden name as `#[link_name = "\u{1}..."]`, i.e. the exact object-file
/// symbol, so LLVM does *not* add the platform's C symbol decoration. Apple targets prefix C
/// symbols with `_`, so `symbol_prefix` must carry it there.
#[derive(Debug)]
struct PrefixedLinkNames {
    symbol_prefix: String,
}

impl ParseCallbacks for PrefixedLinkNames {
    fn generated_link_name_override(&self, item_info: ItemInfo<'_>) -> Option<String> {
        Some(format!("{}{}", self.symbol_prefix, item_info.name))
    }
}

/// Generates raw FFI bindings from the libwebp `encode.h` and `decode.h` headers into
/// `OUT_DIR/bindings.rs`. There is no umbrella header, so both are fed to bindgen.
fn generate_bindings(include_dir: &Path) {
    let encode_h = include_dir.join("encode.h");
    let decode_h = include_dir.join("decode.h");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let symbol_prefix = if env::var("CARGO_CFG_TARGET_VENDOR").unwrap() == "apple" {
        format!("_{SYMBOL_PREFIX}")
    } else {
        SYMBOL_PREFIX.to_string()
    };

    let bindings = bindgen::Builder::default()
        .header(encode_h.to_string_lossy())
        .header(decode_h.to_string_lossy())
        .clang_arg(format!("-I{}", include_dir.display()))
        // Keep the output focused on the libwebp surface.
        .allowlist_function("WebP.*")
        .allowlist_function("VP8.*")
        .allowlist_type("WebP.*")
        .allowlist_type("VP8.*")
        .allowlist_var("WEBP.*")
        .parse_callbacks(Box::new(PrefixedLinkNames { symbol_prefix }))
        .generate_comments(false)
        .layout_tests(false)
        .generate()
        .expect("webp-rs: failed to generate bindings from encode.h/decode.h");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("webp-rs: failed to write bindings.rs");
}
