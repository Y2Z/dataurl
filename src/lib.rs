//! Parsing and generating `data:` URLs.
//!
//! Parsing follows the [data: URL processor] from the WHATWG Fetch Standard, which is what
//! browsers implement (RFC 2397 describes the scheme, but it's not what the web runs on).
//! The implementation is checked against the Web Platform Tests suites for data URLs,
//! forgiving base64, and MIME type parsing.
//!
//! ```
//! use dataurl::DataUrl;
//!
//! let data_url: DataUrl = "data:,Hello,%20World!".parse()?;
//! assert_eq!(data_url.media_type(), "text/plain");
//! assert_eq!(data_url.mime_type(), "text/plain;charset=US-ASCII");
//! assert_eq!(data_url.charset(), "US-ASCII");
//! assert_eq!(data_url.data(), b"Hello, World!");
//! assert_eq!(data_url.text(), "Hello, World!");
//!
//! let mut data_url = DataUrl::new();
//! data_url.set_media_type(Some("image/png"))?;
//! data_url.set_is_base64_encoded(true);
//! data_url.set_data(&[0x89, 0x50, 0x4E, 0x47]);
//! assert_eq!(data_url.to_string(), "data:image/png;base64,iVBORw==");
//! # Ok::<(), dataurl::Error>(())
//! ```
//!
//! [data: URL processor]: https://fetch.spec.whatwg.org/#data-url-processor

mod base64;
mod mime;
mod percent;

use encoding_rs::{Encoding, REPLACEMENT, UTF_16BE, UTF_16LE, WINDOWS_1252};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

const DEFAULT_MEDIA_TYPE: &str = "text/plain";
const DEFAULT_CHARSET: &str = "US-ASCII";
const TEXTUAL_MEDIA_TYPES: &[&str] = &[
    "application/dart",
    "application/ecmascript",
    "application/javascript",
    "application/json",
    "application/jwt",
    "application/x-javascript",
    "application/x-www-form-urlencoded",
    "application/x-yaml",
    "application/xml",
    "application/xml-dtd",
    "application/yaml",
];

/// Errors returned when parsing or modifying a [`DataUrl`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The input doesn't start with the `data:` scheme.
    NotADataUrl,
    /// The input starts with `data:`, but isn't a valid URL (e.g. it has a malformed authority).
    InvalidUrl,
    /// There's no comma separating the metadata from the data.
    MissingComma,
    /// The data is marked as base64-encoded, but isn't valid base64.
    InvalidBase64,
    /// The media type isn't of the `type/subtype` form.
    InvalidMediaType,
    /// The media type parameter has an invalid name or value.
    InvalidParameter,
    /// The charset is unknown, or can't be used for data URLs.
    InvalidCharset,
    /// The text contains characters that the data URL's charset can't represent.
    UnencodableText,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Error::NotADataUrl => "not a data URL",
            Error::InvalidUrl => "invalid URL",
            Error::MissingComma => "data URL has no comma separating metadata from data",
            Error::InvalidBase64 => "data URL contains invalid base64",
            Error::InvalidMediaType => "invalid media type",
            Error::InvalidParameter => "invalid media type parameter",
            Error::InvalidCharset => "invalid or unsupported charset",
            Error::UnencodableText => "text can't be represented in the data URL's charset",
        })
    }
}

impl std::error::Error for Error {}

/// A parsed (or under construction) data URL.
///
/// The data is always kept as raw bytes, exactly as it was decoded or set.
/// Serializing (via [`Display`](fmt::Display), so `to_string()` or `format!`) never transcodes it;
/// use [`set_text`](DataUrl::set_text) to store text in the data URL's charset.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct DataUrl {
    media_type: Option<String>, // Lowercased type/subtype; None if omitted ("text/plain" implied)
    parameters: Vec<(String, String)>, // Media type parameters (charset, etc), names lowercased
    is_base64_encoded: bool,
    data: Vec<u8>,
    fragment: Option<String>,
}

impl fmt::Debug for DataUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Data can be megabytes long, so only its length is shown
        f.debug_struct("DataUrl")
            .field("media_type", &self.media_type)
            .field("parameters", &self.parameters)
            .field("is_base64_encoded", &self.is_base64_encoded)
            .field("data", &format_args!("[{} bytes]", self.data.len()))
            .field("fragment", &self.fragment)
            .finish()
    }
}

impl DataUrl {
    /// Creates an empty data URL (`data:,`).
    pub fn new() -> DataUrl {
        DataUrl::default()
    }

    /// Parses a data URL the way browsers do.
    ///
    /// Like URL parsers, this strips leading and trailing spaces and control characters,
    /// and ignores tabs and newlines anywhere in the input.
    pub fn parse(input: &str) -> Result<DataUrl, Error> {
        let input = input.trim_matches(|c: char| c <= ' ');
        let input: Cow<str> = if input.contains(['\t', '\n', '\r']) {
            Cow::Owned(input.replace(['\t', '\n', '\r'], ""))
        } else {
            Cow::Borrowed(input)
        };

        match input.get(..5) {
            Some(scheme) if scheme.eq_ignore_ascii_case("data:") => {}
            _ => return Err(Error::NotADataUrl),
        }

        let (rest, fragment) = match input[5..].split_once('#') {
            Some((rest, fragment)) => (rest, Some(fragment.to_string())),
            None => (&input[5..], None),
        };

        // Hierarchical forms like "data://host/path" are rare, but they're valid URLs with their own rules
        let rest: Cow<str> = if rest.starts_with('/') {
            Cow::Owned(normalize_hierarchical_part(rest)?)
        } else {
            Cow::Borrowed(rest)
        };

        let (meta_data, encoded_data) = rest.split_once(',').ok_or(Error::MissingComma)?;

        // The metadata part isn't percent-decoded, but URL parsers do percent-encode some of it
        let mut mime_type = String::with_capacity(meta_data.len());
        percent::encode_into(
            &mut mime_type,
            meta_data.as_bytes(),
            percent::in_c0_control_set,
        );
        let mut mime_type: &str = mime_type.trim_matches(|c: char| c.is_ascii_whitespace());

        let mut data: Vec<u8> = percent::decode(encoded_data.as_bytes());

        let is_base64_encoded = match strip_base64_marker(mime_type) {
            Some(stripped) => {
                data = base64::decode_forgiving(&data).ok_or(Error::InvalidBase64)?;
                mime_type = stripped;
                true
            }
            None => false,
        };

        let (media_type, parameters) = if mime_type.starts_with(';') {
            // The media type was omitted, but parameters may be present
            match mime::parse(&format!("{DEFAULT_MEDIA_TYPE}{mime_type}")) {
                // Keep it explicit if there are no parameters, otherwise it'd revert to the default charset
                Some(m) if m.parameters.is_empty() => (Some(m.essence), m.parameters),
                Some(m) => (None, m.parameters),
                None => (None, Vec::new()),
            }
        } else {
            match mime::parse(mime_type) {
                Some(m) => (Some(m.essence), m.parameters),
                None => (None, Vec::new()),
            }
        };

        Ok(DataUrl {
            media_type,
            parameters,
            is_base64_encoded,
            data,
            fragment,
        })
    }

    /// Returns true if the media type isn't a textual one.
    pub fn is_binary(&self) -> bool {
        let media_type: &str = self.media_type();
        let is_textual = media_type.starts_with("text/")
            || media_type.ends_with("+xml")
            || media_type.ends_with("+json")
            || TEXTUAL_MEDIA_TYPES.contains(&media_type);

        !is_textual
    }

    /// The media type's essence (`type/subtype`, lowercase), `"text/plain"` if omitted.
    pub fn media_type(&self) -> &str {
        self.media_type.as_deref().unwrap_or(DEFAULT_MEDIA_TYPE)
    }

    /// The media type's essence, `None` if omitted.
    pub fn media_type_no_default(&self) -> Option<&str> {
        self.media_type.as_deref()
    }

    /// The full MIME type including parameters, the way browsers report it,
    /// e.g. `"text/plain;charset=US-ASCII"` for `data:,`.
    pub fn mime_type(&self) -> String {
        if self.media_type.is_none() && self.parameters.is_empty() {
            return format!("{DEFAULT_MEDIA_TYPE};charset={DEFAULT_CHARSET}");
        }

        let mut mime_type = self.media_type().to_string();
        mime::serialize_parameters(&mut mime_type, &self.parameters);
        mime_type
    }

    /// Sets the media type essence (e.g. `"image/png"`), or removes it with `None`.
    /// Parameters such as charset are kept, and can't be part of the given value.
    pub fn set_media_type(&mut self, media_type: Option<&str>) -> Result<(), Error> {
        match media_type {
            Some(media_type) => {
                let (top_level_type, subtype) = media_type
                    .trim_matches(mime::is_http_whitespace)
                    .split_once('/')
                    .ok_or(Error::InvalidMediaType)?;
                if !mime::is_valid_essence(top_level_type) || !mime::is_valid_essence(subtype) {
                    return Err(Error::InvalidMediaType);
                }
                self.media_type = Some(format!("{top_level_type}/{subtype}").to_ascii_lowercase());
            }
            None => self.media_type = None,
        }

        Ok(())
    }

    /// Iterates over media type parameters as `(name, value)` pairs, in order.
    pub fn parameters(&self) -> impl Iterator<Item = (&str, &str)> {
        self.parameters
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
    }

    /// Returns the value of a media type parameter (names are case-insensitive).
    pub fn parameter(&self, name: &str) -> Option<&str> {
        self.parameters
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Sets or replaces a media type parameter, or removes it with `None`.
    ///
    /// Values may contain any printable ASCII character except `,` and `#`,
    /// which can't be represented in the metadata part of a data URL.
    /// For charset, prefer [`set_charset`](DataUrl::set_charset), which validates the value.
    pub fn set_parameter(&mut self, name: &str, value: Option<&str>) -> Result<(), Error> {
        if !mime::is_valid_essence(name) {
            return Err(Error::InvalidParameter);
        }
        let name = name.to_ascii_lowercase();

        match value {
            Some(value) => {
                if !value
                    .bytes()
                    .all(|b| matches!(b, b' '..=b'~') && b != b',' && b != b'#')
                {
                    return Err(Error::InvalidParameter);
                }
                match self.parameters.iter_mut().find(|(n, _)| *n == name) {
                    Some((_, v)) => *v = value.to_string(),
                    None => self.parameters.push((name, value.to_string())),
                }
            }
            None => self.parameters.retain(|(n, _)| *n != name),
        }

        Ok(())
    }

    /// The charset parameter, `"US-ASCII"` if not present.
    pub fn charset(&self) -> &str {
        self.charset_no_default().unwrap_or(DEFAULT_CHARSET)
    }

    /// The charset parameter, `None` if not present.
    pub fn charset_no_default(&self) -> Option<&str> {
        self.parameter("charset")
    }

    /// Sets the charset parameter, or removes it with `None`.
    ///
    /// Any label from the WHATWG Encoding Standard is accepted, and gets stored as the
    /// encoding's canonical name (e.g. `"utf8"` becomes `"UTF-8"`).
    /// This only changes the label; use [`set_text`](DataUrl::set_text) to encode text in it.
    pub fn set_charset(&mut self, charset: Option<&str>) -> Result<(), Error> {
        match charset {
            Some(label) => {
                let encoding = Encoding::for_label_no_replacement(label.trim().as_bytes())
                    .ok_or(Error::InvalidCharset)?;
                self.set_parameter("charset", Some(encoding.name()))
            }
            None => self.set_parameter("charset", None),
        }
    }

    /// The encoding used for decoding and encoding text, based on the charset parameter.
    /// Missing or unknown charsets result in windows-1252, which is what `US-ASCII` maps to.
    pub fn encoding(&self) -> &'static Encoding {
        self.charset_no_default()
            .and_then(|label| Encoding::for_label(label.as_bytes()))
            .unwrap_or(WINDOWS_1252)
    }

    /// Whether the data gets serialized as base64 (and whether it was, when parsed).
    pub fn is_base64_encoded(&self) -> bool {
        self.is_base64_encoded
    }

    /// Sets whether the data gets serialized as base64 or percent-encoded.
    pub fn set_is_base64_encoded(&mut self, is_base64_encoded: bool) {
        self.is_base64_encoded = is_base64_encoded;
    }

    /// The raw data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Consumes the data URL and returns its raw data.
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// Sets the raw data, which will be serialized byte-for-byte.
    pub fn set_data(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }

    /// The data decoded as text using [`encoding`](DataUrl::encoding).
    /// A byte order mark, if present, takes precedence over the charset.
    pub fn text(&self) -> Cow<'_, str> {
        self.encoding().decode(&self.data).0
    }

    /// Sets the data to the given text, encoded using [`encoding`](DataUrl::encoding).
    ///
    /// Fails if the text contains characters that can't be represented in that encoding
    /// (e.g. emoji in windows-1252, the default); set the charset to UTF-8 to store anything.
    pub fn set_text(&mut self, text: &str) -> Result<(), Error> {
        let encoding = self.encoding();

        self.data = if encoding == UTF_16LE {
            text.encode_utf16().flat_map(u16::to_le_bytes).collect()
        } else if encoding == UTF_16BE {
            text.encode_utf16().flat_map(u16::to_be_bytes).collect()
        } else if encoding == REPLACEMENT {
            return Err(Error::UnencodableText);
        } else {
            let (encoded, _, had_unmappable_characters) = encoding.encode(text);
            if had_unmappable_characters {
                return Err(Error::UnencodableText);
            }
            encoded.into_owned()
        };

        Ok(())
    }

    /// The fragment (the part after `#`), `None` if absent.
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    /// Sets the fragment, or removes it with `None`.
    pub fn set_fragment(&mut self, fragment: Option<&str>) {
        self.fragment = fragment.map(str::to_string);
    }
}

impl fmt::Display for DataUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::from("data:");

        if let Some(media_type) = &self.media_type {
            output.push_str(media_type);
        }
        mime::serialize_parameters(&mut output, &self.parameters);

        if self.is_base64_encoded {
            output.push_str(";base64,");
            base64::encode_into(&mut output, &self.data);
        } else {
            output.push(',');
            percent::encode_into(&mut output, &self.data, percent::in_data_set);
        }

        if let Some(fragment) = &self.fragment {
            output.push('#');
            percent::encode_into(&mut output, fragment.as_bytes(), percent::in_fragment_set);
        }

        f.write_str(&output)
    }
}

impl FromStr for DataUrl {
    type Err = Error;

    fn from_str(input: &str) -> Result<DataUrl, Error> {
        DataUrl::parse(input)
    }
}

impl TryFrom<&str> for DataUrl {
    type Error = Error;

    fn try_from(input: &str) -> Result<DataUrl, Error> {
        DataUrl::parse(input)
    }
}

/// If the metadata ends with ";base64" (optionally with spaces after the semicolon),
/// returns the metadata without it.
fn strip_base64_marker(mime_type: &str) -> Option<&str> {
    let split_at = mime_type.len().checked_sub(6)?;
    if !mime_type.is_char_boundary(split_at) {
        return None;
    }
    let (head, tail) = mime_type.split_at(split_at);

    if !tail.eq_ignore_ascii_case("base64") {
        return None;
    }

    head.trim_end_matches(' ').strip_suffix(';')
}

/// Normalizes the part of a data URL that comes after "data:" when it starts with "/",
/// which makes it a hierarchical URL (with an optional authority) instead of an opaque one.
/// Returns what URL serializers would produce for it, minus the scheme and fragment.
fn normalize_hierarchical_part(input: &str) -> Result<String, Error> {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    if let Some(after_slashes) = input.strip_prefix("//") {
        let authority_end = after_slashes
            .find(['/', '?'])
            .unwrap_or(after_slashes.len());
        let authority = &after_slashes[..authority_end];
        rest = &after_slashes[authority_end..];

        let (userinfo, host_and_port) = match authority.rsplit_once('@') {
            Some((userinfo, host_and_port)) => (Some(userinfo), host_and_port),
            None => (None, authority),
        };

        let (host, port) = match host_and_port.strip_prefix('[') {
            // IPv6 address
            Some(after_bracket) => {
                let end = after_bracket.find(']').ok_or(Error::InvalidUrl)?;
                let address = &after_bracket[..end];
                if address.is_empty()
                    || !address
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
                {
                    return Err(Error::InvalidUrl);
                }
                let after = &after_bracket[end + 1..];
                let port = match after.strip_prefix(':') {
                    Some(port) => port,
                    None if after.is_empty() => "",
                    None => return Err(Error::InvalidUrl),
                };
                (&host_and_port[..end + 2], port)
            }
            None => {
                let (host, port) = host_and_port.split_once(':').unwrap_or((host_and_port, ""));
                if host.contains(['\0', ' ', '<', '>', '@', '[', '\\', ']', '^', '|']) {
                    return Err(Error::InvalidUrl);
                }
                (host, port)
            }
        };

        let port_is_valid = port.is_empty()
            || (port.bytes().all(|b| b.is_ascii_digit()) && port.parse::<u16>().is_ok());
        if !port_is_valid || (host.is_empty() && (userinfo.is_some() || !port.is_empty())) {
            return Err(Error::InvalidUrl);
        }

        output.push_str("//");
        if let Some(userinfo) = userinfo.filter(|u| !u.is_empty()) {
            output.push_str(userinfo);
            output.push('@');
        }
        output.push_str(host);
        if !port.is_empty() {
            output.push(':');
            output.push_str(port);
        }
    }

    // Resolve "." and ".." path segments
    let (path, query) = rest.split_at(rest.find('?').unwrap_or(rest.len()));
    if let Some(path) = path.strip_prefix('/') {
        let is_single_dot = |s: &str| s == "." || s.eq_ignore_ascii_case("%2e");
        let is_double_dot = |s: &str| {
            matches!(
                s.to_ascii_lowercase().as_str(),
                ".." | ".%2e" | "%2e." | "%2e%2e"
            )
        };

        let segments: Vec<&str> = path.split('/').collect();
        let mut resolved: Vec<&str> = Vec::with_capacity(segments.len());
        for (i, segment) in segments.iter().enumerate() {
            let is_last = i == segments.len() - 1;
            if is_double_dot(segment) {
                resolved.pop();
                if is_last {
                    resolved.push("");
                }
            } else if is_single_dot(segment) {
                if is_last {
                    resolved.push("");
                }
            } else {
                resolved.push(segment);
            }
        }

        for segment in resolved {
            output.push('/');
            output.push_str(segment);
        }
    }
    output.push_str(query);

    Ok(output)
}
