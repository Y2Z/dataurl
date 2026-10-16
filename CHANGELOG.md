# Changelog

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
