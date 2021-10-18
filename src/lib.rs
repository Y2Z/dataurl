use encoding_rs::Encoding;
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use std::fmt;
use url::Url;

pub const DEFAULT_MEDIA_TYPE: &'static str = "text/plain";
pub const DEFAULT_CHARSET: &'static str = "US-ASCII";

pub struct DataUrl {
    media_type: Option<String>, // Mime type
    charset: Option<String>,    // US-ASCII is default, according to the spec
    encoded: bool,              // Indicates if it's a base64-encoded data URL
    data: Vec<u8>,              // Data, bytes
    fragment: Option<String>,   // #something-at-the-end, None by default
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

pub fn parse_data_url_meta_data(
    meta_data_string: String,
) -> (Option<String>, Option<String>, bool) {
    let mut media_type: Option<String> = None;
    let mut charset: Option<String> = None;
    let mut encoded: bool = false;

    // Parse meta data
    let content_type_items: Vec<&str> = meta_data_string.split(';').collect();
    let mut i: i8 = 0;
    for item in &content_type_items {
        if i == 0 {
            if item.trim().len() > 0 && item.contains("/") {
                media_type = Some(item.trim().to_string());
            }
        } else {
            if item.trim().eq_ignore_ascii_case("base64") {
                encoded = true;
            } else if item.trim().starts_with("charset=") {
                if let Some(e) = Encoding::for_label_no_replacement((&item[8..]).as_bytes()) {
                    charset = Some(e.name().to_string());
                }
            }
        }

        i += 1;
    }

    (media_type, charset, encoded)
}

impl DataUrl {
    pub fn new() -> DataUrl {
        DataUrl {
            media_type: None,
            charset: None,
            encoded: false,
            data: [].to_vec(),
            fragment: None,
        }
    }

    pub fn parse(input_str: &str) -> Result<Self, DataUrlParseError> {
        match Url::parse(input_str) {
            Ok(url) => {
                let path: String = url.path().to_string();
                if let Some(comma_offset) = path.find(',') {
                    let fragment: Option<&str> = url.fragment();

                    // Parse meta data
                    let meta_data_string = String::from(&path[..comma_offset]);
                    let (media_type, charset, encoded) = parse_data_url_meta_data(meta_data_string);

                    // Parse raw data into vector of bytes
                    let mut data_string: String = percent_decode_str(&path[comma_offset + 1..])
                        .decode_utf8_lossy()
                        .to_string();
                    if let Some(query) = url.query() {
                        data_string += "?";
                        data_string += &percent_decode_str(&query).decode_utf8_lossy().to_string();
                    }
                    let mut unable_to_decode_base64: bool = false;
                    let blob: Vec<u8> = if encoded {
                        match base64::decode(&data_string) {
                            Ok(decoded) => decoded,
                            Err(_) => {
                                unable_to_decode_base64 = true;
                                [].to_vec()
                            }
                        }
                    } else {
                        data_string.as_bytes().to_vec()
                    };

                    if unable_to_decode_base64 {
                        return Err(DataUrlParseError::Base64DecodeError);
                    }

                    Ok(DataUrl {
                        media_type: media_type,
                        charset: charset,
                        encoded: encoded,
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

    pub fn to_string(&self) -> String {
        let mut result: String = "data:".to_string();

        if let Some(mt) = &self.media_type {
            result += &mt;
        }

        if let Some(c) = &self.charset {
            // windows-1252 is same US-ASCII, the default one
            if c != "windows-1252" {
                result += ";charset=";
                result += &c;
            }
        }

        if self.encoded {
            result += ";base64,";
            if self.data.len() > 0 {
                // This can never fail
                if let Some(encoding) = Encoding::for_label(
                    self.charset
                        .as_ref()
                        .unwrap_or(&DEFAULT_CHARSET.to_string())
                        .as_bytes(),
                ) {
                    let (decoded, _, _) = encoding.decode(&self.data);
                    result += &base64::encode(&decoded.as_bytes());
                }
            }
        } else {
            result += ",";
            if self.data.len() > 0 {
                result +=
                    &utf8_percent_encode(&String::from_utf8_lossy(&self.data), NON_ALPHANUMERIC)
                        .to_string();
            }
        }

        if let Some(f) = &self.fragment {
            result += "#";
            result += &utf8_percent_encode(f, NON_ALPHANUMERIC).to_string();
        }

        result
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

    pub fn set_media_type(&mut self, new_media_type: Option<String>) {
        if let Some(mt) = new_media_type {
            if mt.trim().len() > 0 {
                self.media_type = Some(mt.to_string());
            } else {
                // Empty media type makes it fall back to default (text/plain)
                self.media_type = None;
            }
        } else {
            self.media_type = None;
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

    pub fn set_charset(&mut self, new_charset: Option<String>) {
        if let Some(c) = new_charset {
            // Validate the input
            if let Some(e) = Encoding::for_label_no_replacement(c.as_bytes()) {
                self.charset = Some(e.name().to_string());
            } else {
                // Since browsers fall back to US-ASCII, so do we
                self.charset = None;
            }
        } else {
            self.charset = None;
        }
    }

    pub fn encoded(&self) -> bool {
        self.encoded
    }

    pub fn set_encoded(&mut self, new_encoded: bool) {
        self.encoded = new_encoded;
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    // TODO
    // pub fn text(&self) -> String {
    // }

    // TODO
    // pub fn set_text(&self, Option<String>) {
    // }

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
}
