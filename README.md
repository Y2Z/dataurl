[![dataurl build status on GNU/Linux](https://github.com/Y2Z/dataurl/workflows/GNU%2FLinux/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AGNU%2FLinux)
[![dataurl build status on macOS](https://github.com/Y2Z/dataurl/workflows/macOS/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AmacOS)
[![dataurl build status on Windows](https://github.com/Y2Z/dataurl/workflows/Windows/badge.svg)](https://github.com/Y2Z/dataurl/actions?query=workflow%3AWindows)

# dataurl

CLI tool / Rust crate for converting files and text into data URLs and back

Parsing follows the [WHATWG Fetch Standard](https://fetch.spec.whatwg.org/#data-url-processor), which is what browsers implement, and is checked against the [Web Platform Tests](https://github.com/web-platform-tests/wpt) suites for data URLs, base64, and MIME types.
The only dependency is [encoding_rs](https://crates.io/crates/encoding_rs), used for charsets.


---------------------------------------------------


## Usage (CLI)

```console
dataurl "some text"
```

```console
dataurl -d 'data:text/html,text<a id%3D"b">ok</a>?a=v#f' > index.html
```

```console
dataurl -b -t image/png -i picture.png
```

```console
cat file.txt | dataurl -i - -o - | dataurl -d
```

```console
cat file.png | dataurl -b -t image/png
```

### Flags and options

 - `-b`: Encode data using base64
 - `-c`: Use custom `charset` (text given as an argument gets converted to it)
 - `-d`: Attempt to parse input, output resulting data
 - `-f`: Append `fragment`
 - `-i`: Specify `file` to read data from (use `-` for STDIN)
 - `-o`: Provide `file` to write output to (use `-` for STDOUT)
 - `-t`: Adjust `media type`


---------------------------------------------------


## Usage (crate)

```toml
[dependencies]
dataurl = { version = "0.2", default-features = false }
```

```rust
use dataurl::DataUrl;

let data_url: DataUrl = "data:,Hello,%20World!".parse()?;

assert_eq!(data_url.media_type(), "text/plain");
assert_eq!(data_url.media_type_no_default(), None);
assert_eq!(data_url.mime_type(), "text/plain;charset=US-ASCII");
assert_eq!(data_url.charset(), "US-ASCII");
assert_eq!(data_url.charset_no_default(), None);
assert!(!data_url.is_base64_encoded());
assert_eq!(data_url.data(), b"Hello, World!");
assert_eq!(data_url.fragment(), None);
assert_eq!(data_url.text(), "Hello, World!");
assert_eq!(data_url.to_string(), "data:,Hello,%20World!");

let mut data_url = DataUrl::new();
data_url.set_media_type(Some("image/png"))?;
data_url.set_is_base64_encoded(true);
data_url.set_data(&[0x89, 0x50, 0x4E, 0x47]);
assert_eq!(data_url.to_string(), "data:image/png;base64,iVBORw==");
```

Data is stored as raw bytes and serialized as-is; use `set_text()` to store text encoded in the data URL's charset, and `text()` to decode it.
Disabling default features skips the `cli` feature, which only matters for `cargo install`.

If all you need is parsing, [data-url](https://crates.io/crates/data-url) (from the Servo project) is another spec-compliant option;
dataurl also generates data URLs, and comes with a CLI tool.


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
