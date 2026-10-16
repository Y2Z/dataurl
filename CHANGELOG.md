# Changelog

## 0.3.0

### Added

 - `dataurl::mime::MimeType` (also exported as `dataurl::MimeType`): the WHATWG MIME type parser used for
   data URLs, available on its own, e.g. for `Content-Type` headers
 - `DataUrl::set_mime_type()`, which sets the media type and all of its parameters from a `MimeType`
 - `DataEncoding::Shortest`, which picks percent-encoding or base64, whichever results in a shorter data URL
 - `Escaping::Minimal`, which only escapes what has to be when percent-encoding, for data URLs that go inside
   double-quoted HTML attributes or CSS strings; this keeps text formats like SVG short and readable
 - `DataUrl::write_to()`, which writes a data URL to any `std::io::Write` without building it as a string first
 - `DataUrl::encoded_len()`, the exact length of the serialized data URL, e.g. for preallocating buffers
 - `extension_for_media_type()`, the counterpart of `detect_media_type()`, which suggests a file extension
   for a media type (e.g. `svg` for `image/svg+xml`)
 - CLI options:
   - `-s`/`--shortest` and `-m`/`--minimal`, for `DataEncoding::Shortest` and `Escaping::Minimal`
   - `-i` can be given several times, which outputs one data URL per line
   - `--size` prints the lengths of the percent-encoded and base64-encoded data URLs instead of the data URL
   - `-O`/`--output-dir` decodes into files named after the input files, with extensions matching their media types

### Changed

 - `DataUrl::mime_type()` returns a `MimeType` instead of a `String` (use `.to_string()` for the old result)
 - `is_base64_encoded()` and `set_is_base64_encoded()` are replaced by `data_encoding()` and `set_data_encoding()`,
   which use the new `DataEncoding` enum (`Percent`, `Base64` or `Shortest`)
 - Percent-encoding now also escapes `&`, so that HTML entities can't alter the data
   when a data URL is put into an HTML attribute by hand
 - `set_data()` takes `impl Into<Vec<u8>>`: passing a `Vec<u8>` or `String` moves it in without copying,
   while slices, arrays and string literals still work (`&Vec<u8>` needs `.as_slice()` or to be passed by value)
 - Serializing writes in chunks instead of building the whole data URL in memory first, and parsing base64
   no longer makes intermediate copies of the data: for a 10 MB payload, peak memory use while parsing went
   from about 40 MB to 10 MB (just the result), and parsing got about a third faster
 - The CLI writes its output as it encodes, instead of building it in memory first

### Testing

 - CI checks the minimum supported Rust version (1.74), with the newest dependency versions that support it
   (encoding_rs 0.8.40 and newer need Rust 1.88), and runs clippy and rustdoc with warnings treated as errors;
   the stable, beta and nightly builds now really use those toolchains (they all used stable before)
 - Randomized round-trip tests (`tests/roundtrip.rs`) and [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz)
   targets (`fuzz/`), which share the same checks: parsing never panics, and anything that parses or gets set
   survives being serialized and parsed again, with every combination of encoding and escaping

## 0.2.1

### Added

 - `detect_media_type()`, which guesses the media type from the data's signature (using the
   [infer](https://crates.io/crates/infer) crate), falling back to the file extension for text formats
   like CSS and JavaScript, and recognizing SVG either way
 - The CLI detects the media type of files and STDIN input when `-t` isn't given

### Changed

 - Minimum supported Rust version is now 1.74 (required by infer)

## 0.2.0

This release is a rewrite of the parser to follow the WHATWG Fetch Standard (what browsers do),
verified against the Web Platform Tests suites. It fixes several bugs that made 0.1.x unusable
for most real-world data URLs, and cleans up the API, which is why it's a breaking release.

### Fixed

 - Base64 now uses the standard alphabet (`+` and `/`). 0.1.x used the URL-safe one (`-` and `_`),
   so it failed to parse most base64 data URLs, and generated ones browsers couldn't decode
 - Base64 decoding is now forgiving like in browsers: whitespace is ignored and padding is optional
 - Serializing no longer transcodes data: bytes that weren't valid UTF-8 (e.g. windows-1252 text)
   used to get replaced with U+FFFD
 - `charset` is now recognized after whitespace (`data:text/html; charset=utf-8,…`) and when quoted
 - Media type parameters other than `charset` are no longer dropped
 - Inputs with more than 127 parameters no longer cause an integer overflow
 - The CLI no longer ignores `-o` when encoding, and no longer refuses `-o` when STDOUT is redirected
 - Writing into a closed pipe (e.g. `dataurl -d … | head`) no longer panics

### Changed

 - `DataUrlParseError` is replaced by `Error`, which has more precise variants,
   and implements `Display` and `std::error::Error`
 - `to_string()` now comes from `Display`, so `format!("{data_url}")` works too
 - Getters return borrowed values: `media_type_no_default()`, `charset_no_default()`
   and `fragment()` return `Option<&str>`, and `text()` returns `Cow<str>`
 - Setters take `Option<&str>`; `set_media_type()` and `set_charset()` return `Result<(), Error>`
   instead of `bool`
 - `set_media_type()` only accepts `type/subtype`; use `set_parameter()` for parameters
 - Percent-encoded output is much more readable, only escaping what URLs, HTML attributes and CSS require
   (`data:,Hello,%20world!` instead of `data:,Hello%2C%20world%21`)
 - When given text as an argument together with `-c`, the CLI fails if the text can't be represented
   in that charset (it used to output HTML numeric character references like `&#220;`)
 - CLI error messages are more descriptive, and usage errors exit with code 2
 - Updated to Rust 2021 edition, minimum supported Rust version is 1.73

### Added

 - `FromStr` and `TryFrom<&str>`, so `"data:,…".parse::<DataUrl>()` works
 - `Debug`, `Clone`, `PartialEq`, `Eq` and `Default` for `DataUrl` (`Debug` doesn't print the data, only its length)
 - `mime_type()`, the full MIME type the way browsers report it (e.g. `text/plain;charset=US-ASCII`)
 - `parameters()`, `parameter()` and `set_parameter()` for media type parameters
 - `encoding()`, the `encoding_rs` encoding used for text
 - `set_text()`, which encodes text in the data URL's charset
 - `into_data()`
 - Support for `--option=value` and `-ovalue` forms in the CLI

### Removed

 - Dependencies on `atty` (unmaintained, RUSTSEC-2021-0145), `clap`, `base64`, `percent-encoding` and `url`;
   the CLI and the library now only depend on `encoding_rs`
