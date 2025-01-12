use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use encoding_rs::Encoding;
use percent_encoding::{percent_decode_str, percent_encode, utf8_percent_encode, NON_ALPHANUMERIC};
use std::fmt;
use url::Url;

const DEFAULT_MEDIA_TYPE: &'static str = "text/plain";
const DEFAULT_CHARSET: &'static str = "US-ASCII";
const FILE_SIGNATURES: [[&[u8]; 2]; 18] = [
    // Image
    [b"GIF87a", b"image/gif"],
    [b"GIF89a", b"image/gif"],
    [b"\xFF\xD8\xFF", b"image/jpeg"],
    [b"\x89PNG\x0D\x0A\x1A\x0A", b"image/png"],
    [b"<svg ", b"image/svg+xml"],
    [b"RIFF....WEBPVP8 ", b"image/webp"],
    [b"\x00\x00\x01\x00", b"image/x-icon"],
    // Audio
    [b"ID3", b"audio/mpeg"],
    [b"\xFF\x0E", b"audio/mpeg"],
    [b"\xFF\x0F", b"audio/mpeg"],
    [b"OggS", b"audio/ogg"],
    [b"RIFF....WAVEfmt ", b"audio/wav"],
    [b"fLaC", b"audio/x-flac"],
    // Video
    [b"RIFF....AVI LIST", b"video/avi"],
    [b"....ftyp", b"video/mp4"],
    [b"\x00\x00\x01\x0B", b"video/mpeg"],
    [b"....moov", b"video/quicktime"],
    [b"\x1A\x45\xDF\xA3", b"video/webm"],
];
const PLAINTEXT_MEDIA_TYPES: &'static [&str] = &[
    "application/atom+xml",
    "application/dart",
    "application/ecmascript",
    "application/javascript",
    "application/json",
    "application/jwt",
    "application/rdf+xml",
    "application/rss+xml",
    "application/soap+xml",
    "application/vnd.mozilla.xul+xml",
    "application/x-javascript",
    "application/x-yaml",
    "application/xhtml+xml",
    "application/xml",
    "application/xml-dtd",
    "application/xop+xml",
    "application/yaml",
    "image/svg+xml",
    "message/imdn+xml",
    "model/x3d+xml",
];

// TODO: add support for other optional parameters besides charset (filename, etc)
pub struct DataUrl {
    media_type: Option<String>, // Media type
    charset: Option<String>,    // US-ASCII is default, according to the spec
    is_base64_encoded: bool,    // Indicates if it's a base64-encoded data URL
    data: Vec<u8>,              // Data, bytes, UTF-8 if text
    fragment: Option<String>,   // #something at the end, None by default
}

pub enum DataUrlParseError {
    UrlParseError,
    MalformedDataUrlError,
    Base64DecodeError,
}

impl fmt::Debug for DataUrlParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DataUrlParseError").finish()
    }
}

pub fn detect_media_type(data: &[u8], filename: &str) -> String {
    // At first attempt to read file's header
    for file_signaure in FILE_SIGNATURES.iter() {
        if data.starts_with(file_signaure[0]) {
            return String::from_utf8(file_signaure[1].to_vec()).unwrap();
        }
    }

    // If header didn't match any known magic signatures,
    // try to guess media type from file name
    detect_media_type_by_file_name(&filename)
}

pub fn detect_media_type_by_file_name(filename: &str) -> String {
    let filename_lowercased: &str = &filename.to_lowercase();
    let parts: Vec<&str> = filename_lowercased.split('.').collect();

    let mime: &str = match parts.last() {
        Some(v) => match *v {
            "avi" => "video/avi",
            "bmp" => "image/bmp",
            "css" => "text/css",
            "flac" => "audio/flac",
            "gif" => "image/gif",
            "htm" | "html" => "text/html",
            "ico" => "image/x-icon",
            "jpeg" | "jpg" => "image/jpeg",
            "js" => "application/javascript",
            "json" => "application/json",
            "mp3" => "audio/mpeg",
            "mp4" | "m4v" => "video/mp4",
            "ogg" => "audio/ogg",
            "ogv" => "video/ogg",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "svg" => "image/svg+xml",
            "swf" => "application/x-shockwave-flash",
            "tif" | "tiff" => "image/tiff",
            "txt" => "text/plain",
            "wav" => "audio/wav",
            "webp" => "image/webp",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "xml" => "text/xml",
            &_ => "",
        },
        None => "",
    };

    mime.to_string()
}

pub(crate) fn parse_data_url_meta_data(
    meta_data_string: String,
) -> (Option<String>, Option<String>, bool) {
    let mut media_type: Option<String> = None;
    let mut charset: Option<String> = None;
    let mut is_base64_encoded: bool = false;

    // Parse meta data
    let content_type_items: Vec<&str> = meta_data_string.split(';').collect();
    let mut i: i8 = 0;
    for item in &content_type_items {
        // Media type has to always come first in data URLs
        if i == 0 {
            if item.trim().len() > 0 && validate_media_type(item) {
                media_type = Some(item.trim().to_lowercase().to_string());
            }
        } else {
            if !is_base64_encoded && item.trim().to_lowercase().starts_with("charset=") {
                // only the first occurence of charset counts
                if charset.is_none() {
                    if let Some(e) = Encoding::for_label_no_replacement((&item[8..]).as_bytes()) {
                        charset = Some(e.name().to_string());
                    }
                }
            } else if item.trim().eq_ignore_ascii_case("base64") {
                is_base64_encoded = true;
            }
        }

        i += 1;
    }

    (media_type, charset, is_base64_encoded)
}

pub(crate) fn validate_media_type(media_type: &str) -> bool {
    // Must contain one forward slash
    media_type.split('/').collect::<Vec<&str>>().len() == 2
}

impl DataUrl {
    pub fn new() -> DataUrl {
        DataUrl {
            media_type: None,
            charset: None,
            is_base64_encoded: false,
            data: [].to_vec(),
            fragment: None,
        }
    }

    // TODO: rename to from_string/from_str/from — look for how it's done for String and similar
    pub fn parse(input_str: &str) -> Result<Self, DataUrlParseError> {
        match Url::parse(input_str) {
            Ok(url) => {
                let path: String = url.path().to_string();
                if let Some(comma_offset) = path.find(',') {
                    let fragment: Option<&str> = url.fragment();

                    // Parse meta data
                    let meta_data_string = String::from(&path[..comma_offset]);
                    let (media_type, charset, is_base64_encoded) =
                        parse_data_url_meta_data(meta_data_string);

                    // Parse raw data into vector of bytes
                    let mut d: Vec<u8> = percent_decode_str(&path[comma_offset + 1..]).collect();
                    if let Some(query) = url.query() {
                        d.push("?".as_bytes()[0]);
                        d.append(&mut percent_decode_str(&query).collect());
                    }
                    let mut unable_to_decode_base64: bool = false;
                    let blob: Vec<u8> = if is_base64_encoded {
                        match URL_SAFE.decode(&d) {
                            Ok(decoded) => decoded,
                            Err(_) => {
                                unable_to_decode_base64 = true;
                                [].to_vec()
                            }
                        }
                    } else {
                        d
                    };

                    if unable_to_decode_base64 {
                        return Err(DataUrlParseError::Base64DecodeError);
                    }

                    Ok(DataUrl {
                        media_type: media_type,
                        charset: charset,
                        is_base64_encoded: is_base64_encoded,
                        data: blob,
                        fragment: if let Some(f) = fragment {
                            Some(f.to_string())
                        } else {
                            None
                        },
                    })
                } else {
                    Err(DataUrlParseError::MalformedDataUrlError)
                }
            }
            Err(_) => Err(DataUrlParseError::UrlParseError),
        }
    }

    pub fn is_binary(&self) -> bool {
        if self.media_type.is_none() {
            return false;
        }

        let current_media_type: &str = &self.media_type.as_ref().unwrap();
        let is_plaintext: bool = if current_media_type.split('/').collect::<Vec<&str>>()[0]
            .eq_ignore_ascii_case("text")
        {
            true
        } else {
            PLAINTEXT_MEDIA_TYPES
                .iter()
                .find(|mt| current_media_type.eq_ignore_ascii_case(mt))
                .is_some()
        };

        !is_plaintext
    }

    pub fn media_type(&self) -> &str {
        if let Some(mt) = &self.media_type {
            mt
        } else {
            DEFAULT_MEDIA_TYPE
        }
    }

    pub fn media_type_no_default(&self) -> Option<String> {
        if let Some(mt) = &self.media_type {
            Some(mt.to_string())
        } else {
            None
        }
    }

    pub fn set_media_type(&mut self, new_media_type: Option<String>) -> bool {
        if let Some(mt) = new_media_type {
            if mt.trim().len() > 0 && validate_media_type(&mt) {
                self.media_type = Some(mt.to_string());
                true
            } else {
                // Empty media type makes it fall back to default (text/plain)
                self.media_type = None;
                false
            }
        } else {
            self.media_type = None;
            true
        }
    }

    pub fn charset(&self) -> &str {
        if let Some(c) = &self.charset {
            c
        } else {
            DEFAULT_CHARSET
        }
    }

    pub fn charset_no_default(&self) -> Option<String> {
        if let Some(c) = &self.charset {
            Some(c.to_string())
        } else {
            None
        }
    }

    pub fn set_charset(&mut self, new_charset: Option<String>) -> bool {
        if let Some(nc) = new_charset {
            // Validate the input
            if let Some(e) = Encoding::for_label_no_replacement(nc.as_bytes()) {
                self.charset = Some(e.name().to_string());
                true
            } else {
                // Since browsers fall back to US-ASCII, so does this
                self.charset = None;
                false
            }
        } else {
            // Unset
            self.charset = None;
            true
        }
    }

    // TODO: ditch get/set_is_base64_encode and implement two separate functions, to_precent_encoded_string, and to_base64_encoded_string?
    // TODO: ^ if taken that path, should was_input_base64_encoded() added, None by default, Option<bool> after parse() is used, added?

    pub fn is_base64_encoded(&self) -> bool {
        self.is_base64_encoded
    }

    pub fn set_is_base64_encoded(&mut self, new_is_base64_encoded: bool) {
        self.is_base64_encoded = new_is_base64_encoded;
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn text(&self) -> String {
        // This can never really fail
        if let Some(encoding) = Encoding::for_label_no_replacement(
            self.charset
                .as_ref()
                .unwrap_or(&DEFAULT_CHARSET.to_string())
                .as_bytes(),
        ) {
            let (decoded, _, _) = encoding.decode(&self.data);
            decoded.to_string()
        } else {
            "".to_string()
        }
    }

    /*
        // TODO: add new_text_charset argument?
        pub fn set_text(&mut self, new_text: &str) {
            if self.charset == Some("UTF-8".to_string()) {
                self.data = new_text.as_bytes().to_vec();
            } else {
                if let Some(encoding) = Encoding::for_label_no_replacement(
                    self.charset
                        .as_ref()
                        .unwrap_or(&DEFAULT_CHARSET.to_string())
                        .as_bytes(),
                ) {
                    let (decoded, _, _) = encoding.decode(&new_text.as_bytes());
                    self.data = decoded.as_bytes().to_vec();
                }
            }
        }
    */

    pub fn set_data(&mut self, new_data: &[u8]) {
        self.data = new_data.to_vec();
    }

    pub fn fragment(&self) -> Option<String> {
        if let Some(f) = &self.fragment {
            Some(f.to_string())
        } else {
            None
        }
    }

    pub fn set_fragment(&mut self, new_fragment: Option<String>) {
        self.fragment = new_fragment;
    }

    // TODO: rename it to as_str/to_str, make it return a &str instead of String
    // TODO: make it an Option(Result?), throw error in case is_base64_encoded=false, and charset!=default|utf8
    pub fn to_string(&self) -> String {
        let mut result: String = String::from("data:");

        if let Some(mt) = &self.media_type {
            result += &mt;
        }

        if let Some(c) = &self.charset {
            // NOTE: windows-1252 is another name for US-ASCII, the default charset for data URLs
            if c != "windows-1252" {
                result += ";charset=";
                result += &c;
            }
        }

        {
            if self.is_base64_encoded {
                result += ";base64";
            }
            result += ",";

            if self.data.len() > 0 {
                if self.is_binary() {
                    // Just encode as base64 or URI if data is binary
                    if self.is_base64_encoded {
                        result += &URL_SAFE.encode(&self.data);
                    } else {
                        result += &percent_encode(&self.data, NON_ALPHANUMERIC).to_string();
                    }
                } else {
                    // Charset only matters for textual data
                    let data_as_utf8_string: String =
                        String::from_utf8_lossy(&self.data).to_string();
                    let fallback_charset: String = if data_as_utf8_string.is_ascii() {
                        DEFAULT_CHARSET.to_string()
                    } else {
                        "UTF-8".to_string()
                    };

                    if let Some(encoding) = Encoding::for_label_no_replacement(
                        self.charset
                            .as_ref()
                            .unwrap_or(&fallback_charset)
                            .as_bytes(),
                    ) {
                        let (encoded, _, _) = encoding.encode(&data_as_utf8_string);

                        if self.is_base64_encoded {
                            result += &URL_SAFE.encode(&encoded.to_vec());
                        } else {
                            result +=
                                &percent_encode(&encoded.to_vec(), NON_ALPHANUMERIC).to_string();
                        }
                    }
                }
            }
        }

        if let Some(f) = &self.fragment {
            result += "#";
            // TODO: need to deal with encoding here as well
            result += &utf8_percent_encode(f, NON_ALPHANUMERIC).to_string();
        }

        result
    }
}
