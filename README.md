# webp-rs

A Rust library to encode and decode WebP images via statically-linked libwebp.

## ⬇️ Installation

This library can be installed using Cargo. To do that, run the following command in your project's root directory:

```bash
cargo add webp-rs
```

The crate links as `webp`, so you import it with `use webp;` regardless of the package name.

> [!NOTE]
> The first build downloads the prebuilt static binaries for your platform, so an internet connection is required (see [Troubleshooting](#-troubleshooting) for offline builds).

## 🤖 Usage

Here are some examples of how to encode and decode WebP images using this library. These snippets don't have any error handling for the sake of simplicity, but you should always check for errors in production code.

#### Encoding

```rust
let img = image::open("/path/to/image.png").unwrap(); // an image to be encoded
let bytes = webp::encode(&img).unwrap(); // encode the image with default settings
std::fs::write("/path/to/image.webp", &bytes).unwrap(); // save the WebP to a file
```

#### Encoding with custom settings

```rust
use webp::WebpEncoder;

let img = image::open("/path/to/image.png").unwrap();
let mut bytes = Vec::new();
img.write_with_encoder(
    WebpEncoder::new(&mut bytes)
        .with_quality(80)      // 0–100, higher = better quality
        .with_compression(6)   // 0–6, higher = more effort, smaller file
        .with_threads(true),   // enable multi-threaded encoding
).unwrap();
```

#### Lossless encoding

```rust
use webp::WebpEncoder;

let img = image::open("/path/to/image.png").unwrap();
let mut bytes = Vec::new();
img.write_with_encoder(WebpEncoder::new(&mut bytes).with_lossless(true)).unwrap();
```

#### Decoding

```rust
let bytes = std::fs::read("/path/to/image.webp").unwrap(); // read the WebP file
let img = webp::decode(&bytes).unwrap(); // decode it into a DynamicImage
img.save("/path/to/image.png").unwrap(); // save it in another format
```

#### Probing (header only)

Read the image dimensions and color type without decoding the pixels — useful for validation or thumbnailing pipelines:

```rust
let bytes = std::fs::read("/path/to/image.webp").unwrap();
let info = webp::probe(&bytes).unwrap();
println!("{}x{} ({:?})", info.width, info.height, info.color_type);
```

The public API also exposes [`encode_buffer`] (encode a typed `ImageBuffer` directly), [`WebpEncoder`] / [`WebpDecoder`] for `image`-trait integration, [`EncoderConfig`] / [`DecoderConfig`] for full control, and [`libwebp_version`].

#### Encoder settings

| Setting              | Range / type | Default | Meaning                                                                 |
|----------------------|--------------|---------|-------------------------------------------------------------------------|
| `with_quality`       | `0–100`      | `75`    | Visual-quality target (higher = better, larger). Ignored when lossless. |
| `with_quality_alpha` | `0–100`      | `100`   | Quality of the alpha channel.                                           |
| `with_compression`   | `0–6`        | `4`     | Compression effort (higher = slower encode, smaller file).              |
| `with_lossless`      | `bool`       | `false` | Lossless encoding (reconstructs pixels exactly).                        |
| `with_threads`       | `bool`       | `false` | Enable libwebp's multi-threaded encoding.                               |

> WebP is an 8-bit format: inputs are encoded as 8-bit RGB/RGBA (grayscale is expanded automatically) and decoded back to the same. 16-bit inputs are rejected as unsupported.

#### Runnable examples

The [`examples/`](examples) directory has standalone programs covering each part of the API, runnable out of the box against the bundled assets:

```bash
cargo run --example encode          # encode with defaults
cargo run --example decode          # decode a WebP to PNG
cargo run --example custom_encoder  # WebpEncoder builder (quality/compression/threads)
cargo run --example encode_buffer   # encode a typed ImageBuffer
cargo run --example probe           # read the header without decoding pixels
cargo run --example roundtrip       # encode then decode
cargo run --example lossless        # lossless encoding via EncoderConfig
cargo run --example parallel_encode # concurrent encoding
cargo run --example version         # print the linked libwebp version
```

## 💣 Troubleshooting

### My build fails because it can't download the binaries

The first build fetches the prebuilt static libraries for your platform over the network. For offline or air-gapped builds, download the archive for your target from [binaries-webp](https://github.com/vegidio/binaries-webp/releases), extract it, and point the build at it with the `WEBP_BINARIES_DIR` environment variable:

```
$ WEBP_BINARIES_DIR=/path/to/extracted/libs cargo build
```

## 📝 License

**webp-rs** is released under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

## 👨🏾‍💻 Author

Vinicius Egidio ([vinicius.io](http://vinicius.io))
