[![dataurl on crates.io](https://img.shields.io/crates/v/dataurl.svg)](https://crates.io/crates/dataurl)
[![dataurl documentation on docs.rs](https://docs.rs/dataurl/badge.svg)](https://docs.rs/dataurl)
[![dataurl build status on GNU/Linux](https://github.com/Y2Z/dataurl/actions/workflows/build_gnu_linux.yml/badge.svg)](https://github.com/Y2Z/dataurl/actions/workflows/build_gnu_linux.yml)
[![dataurl build status on macOS](https://github.com/Y2Z/dataurl/actions/workflows/build_macos.yml/badge.svg)](https://github.com/Y2Z/dataurl/actions/workflows/build_macos.yml)
[![dataurl build status on Windows](https://github.com/Y2Z/dataurl/actions/workflows/build_windows.yml/badge.svg)](https://github.com/Y2Z/dataurl/actions/workflows/build_windows.yml)

# dataurl

CLI tool / Rust crate for converting files and text into data URLs and back

Parsing follows the [WHATWG Fetch Standard](https://fetch.spec.whatwg.org/#data-url-processor), which is what browsers implement, and is checked against the [Web Platform Tests](https://github.com/web-platform-tests/wpt) suites for data URLs, base64, and MIME types.
Dependencies are kept minimal: [encoding_rs](https://crates.io/crates/encoding_rs) for charsets, and [infer](https://crates.io/crates/infer) for detecting media types (both have no dependencies of their own here).


---------------------------------------------------


## Usage (CLI)

```console
dataurl "some text"
```

```console
dataurl -d 'data:text/html,text<a id%3D"b">ok</a>?a=v#f' > index.html
```

```console
dataurl -b -i picture.png
```

```console
cat file.txt | dataurl -i - -o - | dataurl -d
```

```console
cat file.png | dataurl -b
```

```console
dataurl -s -m -i logo.svg -i photo.jpg -i font.woff2 > urls.txt
```

```console
dataurl --size -i logo.svg -i photo.jpg
```

```console
dataurl -d -O assets/ -i logo.txt -i photo.txt
```

### Flags and options

 - `-b`: Encode data using base64
 - `-s`: Use base64 or percent-encoding, whichever results in a shorter data URL
 - `-m`: Only escape what has to be when percent-encoding (for data URLs that go inside double-quoted HTML attributes or CSS strings)
 - `-c`: Use custom `charset` (text given as an argument gets converted to it)
 - `-f`: Append `fragment`
 - `-t`: Adjust `media type` (detected automatically for files and STDIN)
 - `-i`: Specify `file` to read data from (use `-` for STDIN); can be given several times, for one data URL per line
 - `-o`: Provide `file` to write output to (use `-` for STDOUT)
 - `--size`: Print the lengths of the percent-encoded and base64-encoded data URLs instead, tab-separated, followed by the file name (if any)
 - `-d`: Attempt to parse input, output resulting data
 - `-O`: Decode into files in `directory`, named after the input files, with extensions matching their media types (the names written get printed)


---------------------------------------------------


## Usage (crate)

```toml
[dependencies]
dataurl = { version = "0.3", default-features = false }
```

```rust
use dataurl::{DataEncoding, DataUrl};

let data_url: DataUrl = "data:,Hello,%20World!".parse()?;

assert_eq!(data_url.media_type(), "text/plain");
assert_eq!(data_url.media_type_no_default(), None);
assert_eq!(data_url.mime_type().to_string(), "text/plain;charset=US-ASCII");
assert_eq!(data_url.charset(), "US-ASCII");
assert_eq!(data_url.charset_no_default(), None);
assert_eq!(data_url.data_encoding(), DataEncoding::Percent);
assert_eq!(data_url.data(), b"Hello, World!");
assert_eq!(data_url.fragment(), None);
assert_eq!(data_url.text(), "Hello, World!");
assert_eq!(data_url.to_string(), "data:,Hello,%20World!");

let mut data_url = DataUrl::new();
data_url.set_media_type(Some("image/png"))?;
data_url.set_data_encoding(DataEncoding::Base64);
data_url.set_data([0x89, 0x50, 0x4E, 0x47]);
assert_eq!(data_url.to_string(), "data:image/png;base64,iVBORw==");
```

Data is stored as raw bytes and serialized as-is; use `set_text()` to store text encoded in the data URL's charset, and `text()` to decode it.
Disabling default features skips the `cli` feature, which only matters for `cargo install`.

The minimum supported Rust version is 1.74. Recent versions of encoding_rs (0.8.40 and newer) need Rust 1.88,
so on older toolchains, have Cargo pick dependency versions that match yours:
`CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback cargo update` (with Cargo 1.84 or newer).

### Choosing how data gets encoded

`set_data_encoding()` picks percent-encoding (the default), base64, or `DataEncoding::Shortest`, whichever comes out shorter for the data.
`set_escaping()` controls how much percent-encoding escapes: `Escaping::Full` (the default) makes the data URL safe anywhere,
while `Escaping::Minimal` only escapes what has to be, which keeps text formats like SVG short and readable,
for data URLs that go inside double-quoted HTML attributes or CSS strings:

```rust
use dataurl::{DataEncoding, DataUrl, Escaping};

let mut data_url = DataUrl::new();
data_url.set_media_type(Some("image/svg+xml"))?;
data_url.set_data_encoding(DataEncoding::Shortest);
data_url.set_escaping(Escaping::Minimal);
data_url.set_data(b"<svg xmlns='http://www.w3.org/2000/svg'><circle r='4'/></svg>");

assert_eq!(
    data_url.to_string(),
    "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'%3E%3Ccircle r='4'/%3E%3C/svg%3E"
);
```

### MIME types

The WHATWG MIME type parser used for data URLs is available on its own, e.g. for `Content-Type` headers:

```rust
use dataurl::{DataUrl, MimeType};

let mime_type: MimeType = "Text/CSS; charset=\"UTF-8\"".parse()?;
assert_eq!(mime_type.essence(), "text/css");
assert_eq!(mime_type.charset(), Some("UTF-8"));

let mut data_url = DataUrl::new();
data_url.set_mime_type(Some(&mime_type))?;
assert_eq!(data_url.to_string(), "data:text/css;charset=UTF-8,");
```

### Media type detection

Media types can be detected from the data's signature, falling back to the file extension
(which is what text formats like CSS and JavaScript need):

```rust
use dataurl::detect_media_type;

assert_eq!(detect_media_type(b"\x89PNG\r\n\x1a\n", None), Some("image/png"));
assert_eq!(detect_media_type(b"body{}", Some("style.css")), Some("text/css"));
```

`extension_for_media_type()` goes the other way, e.g. for saving decoded data to files:

```rust
use dataurl::extension_for_media_type;

assert_eq!(extension_for_media_type("image/svg+xml"), Some("svg"));
assert_eq!(extension_for_media_type("application/javascript"), Some("js"));
```

The full API is documented on [docs.rs](https://docs.rs/dataurl).

### Serde

With the `serde` feature, `DataUrl` and `MimeType` implement `Serialize` and `Deserialize`, as strings,
so they can be part of configuration files, API responses and so on:

```toml
[dependencies]
dataurl = { version = "0.3", default-features = false, features = ["serde"] }
```

```rust
use dataurl::DataUrl;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Theme {
    name: String,
    logo: DataUrl, // "data:image/svg+xml,..." in JSON, TOML, YAML…
}
```

Invalid data URLs and MIME types make deserializing fail.

### Comparison with data-url

[data-url](https://crates.io/crates/data-url), from the Servo project (it lives in the [rust-url](https://github.com/servo/rust-url) repository),
is the other spec-compliant data URL crate. Both follow the WHATWG Fetch Standard and pass the Web Platform Tests, but they're built for different jobs:

|                                              | dataurl                                       | data-url                                  |
|----------------------------------------------|-----------------------------------------------|-------------------------------------------|
| Parsing (WHATWG Fetch Standard)              | Yes                                           | Yes                                       |
| Generating data URLs                         | Yes: percent-encoding, base64, or shortest    | No                                        |
| Changing media type, charset, fragment, data | Yes                                           | No                                        |
| Decoding text using the charset              | Yes (encoding_rs)                             | No, bytes only                            |
| Media type detection                         | Yes (infer, plus file extensions)             | No                                        |
| MIME type parser                             | Yes                                           | Yes                                       |
| Forgiving-base64 decoder on its own          | No                                            | Yes                                       |
| Decoding                                     | Into a `Vec<u8>`                              | Into a `Vec<u8>`, or streamed to a callback without copying the input |
| `no_std`                                     | No                                            | Yes (with `alloc`)                        |
| Dependencies                                 | encoding_rs, infer                            | None                                      |
| Minimum Rust version                         | 1.74                                          | 1.51                                      |
| CLI tool                                     | Yes                                           | No                                        |
| License                                      | CC0 (public domain)                           | MIT or Apache-2.0                         |

If you only need to read data URLs, especially in `no_std` code, or you already use the [url](https://crates.io/crates/url) crate, data-url is the leaner choice.
If you need to create or modify data URLs, work with text in other charsets, or figure out what kind of file the data is, that's what dataurl is for.


---------------------------------------------------


## Testing

```console
make test
```

Besides unit tests (including the ones for the `serde` feature), this runs the [Web Platform Tests](https://github.com/web-platform-tests/wpt) suites for data URLs,
base64 and MIME types (in `tests/wpt/`), and randomized round-trip tests (`tests/roundtrip.rs`).
Those run 5000 iterations each by default; set `DATAURL_ROUNDTRIP_ITERATIONS` for more.
The same checks are available as [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets, which need a nightly toolchain:

```console
cargo install cargo-fuzz
cargo +nightly fuzz run parse
cargo +nightly fuzz run serialize
```


---------------------------------------------------


## Installation

#### Using [Cargo](https://crates.io/crates/dataurl)

```console
cargo install dataurl
```

#### Using [containers](https://www.docker.com/)

```console
docker build -t Y2Z/dataurl .
sudo install -b dist/run-in-container.sh /usr/local/bin/dataurl
```

#### From source

```console
git clone https://github.com/Y2Z/dataurl.git
cd dataurl
make install
```

#### Using [pre-built binaries](https://github.com/Y2Z/dataurl/releases) (Windows, ARM-based devices, etc)

Every release contains pre-built binaries for Windows, GNU/Linux, as well as platforms with non-standard CPU architecture.


---------------------------------------------------


## References

 - [WHATWG Fetch Standard: data: URL processor](https://fetch.spec.whatwg.org/#data-url-processor)
 - [WHATWG MIME Sniffing Standard: parsing a MIME type](https://mimesniff.spec.whatwg.org/#parsing-a-mime-type)
 - [WHATWG Infra Standard: forgiving-base64 decode](https://infra.spec.whatwg.org/#forgiving-base64-decode)
 - [RFC 2397 (The "data" URL scheme)](https://datatracker.ietf.org/doc/html/rfc2397)
 - [RFC 6838 (Media Type Specifications and Registration Procedures)](https://datatracker.ietf.org/doc/html/rfc6838)


---------------------------------------------------


## License

To the extent possible under law, the author(s) have dedicated all copyright related and neighboring rights to this software to the public domain worldwide.
This software is distributed without any warranty.

The Web Platform Tests fixtures in `tests/wpt/` are distributed under their own [3-Clause BSD License](tests/wpt/README.md).
