//! MIME type parsing and serialization, as defined by the WHATWG MIME Sniffing
//! Standard (<https://mimesniff.spec.whatwg.org/#parsing-a-mime-type>).

pub(crate) type Parameters = Vec<(String, String)>;

pub(crate) struct MimeType {
    pub essence: String, // Lowercased "type/subtype"
    pub parameters: Parameters,
}

pub(crate) fn is_http_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | ' ')
}

pub(crate) fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c)
}

fn is_quoted_string_token_char(c: char) -> bool {
    matches!(c, '\t' | ' '..='~' | '\u{80}'..='\u{ff}')
}

pub(crate) fn is_valid_essence(type_or_subtype: &str) -> bool {
    !type_or_subtype.is_empty() && type_or_subtype.chars().all(is_token_char)
}

/// Returns None if the input is not a valid MIME type.
pub(crate) fn parse(input: &str) -> Option<MimeType> {
    let s = input.trim_matches(is_http_whitespace);
    let b = s.as_bytes();

    let (top_level_type, rest) = s.split_once('/')?;
    if !is_valid_essence(top_level_type) {
        return None;
    }

    let subtype_end = rest.find(';').unwrap_or(rest.len());
    let subtype = rest[..subtype_end].trim_end_matches(is_http_whitespace);
    if !is_valid_essence(subtype) {
        return None;
    }

    let mut mime_type = MimeType {
        essence: format!(
            "{}/{}",
            top_level_type.to_ascii_lowercase(),
            subtype.to_ascii_lowercase()
        ),
        parameters: Vec::new(),
    };

    // All delimiters are ASCII, so byte offsets always land on char boundaries
    let mut pos = top_level_type.len() + 1 + subtype_end;
    while pos < b.len() {
        // Skip past the semicolon and any whitespace after it
        pos += 1;
        while pos < b.len() && is_http_whitespace(b[pos] as char) {
            pos += 1;
        }

        let name_start = pos;
        while pos < b.len() && b[pos] != b';' && b[pos] != b'=' {
            pos += 1;
        }
        let name = s[name_start..pos].to_ascii_lowercase();

        if pos < b.len() {
            if b[pos] == b';' {
                continue;
            }
            pos += 1; // Skip past "="
        }
        if pos >= b.len() {
            break;
        }

        let value: String = if b[pos] == b'"' {
            let (value, new_pos) = collect_http_quoted_string(s, pos);
            pos = new_pos;
            while pos < b.len() && b[pos] != b';' {
                pos += 1;
            }
            value
        } else {
            let value_start = pos;
            while pos < b.len() && b[pos] != b';' {
                pos += 1;
            }
            let value = s[value_start..pos].trim_end_matches(is_http_whitespace);
            if value.is_empty() {
                continue;
            }
            value.to_string()
        };

        if !name.is_empty()
            && name.chars().all(is_token_char)
            && value.chars().all(is_quoted_string_token_char)
            && !mime_type.parameters.iter().any(|(n, _)| *n == name)
        {
            mime_type.parameters.push((name, value));
        }
    }

    Some(mime_type)
}

// https://fetch.spec.whatwg.org/#collect-an-http-quoted-string (with extract-value set to true)
fn collect_http_quoted_string(s: &str, mut pos: usize) -> (String, usize) {
    let b = s.as_bytes();
    let mut value = String::new();

    pos += 1; // Skip past the opening quote
    loop {
        let start = pos;
        while pos < b.len() && b[pos] != b'"' && b[pos] != b'\\' {
            pos += 1;
        }
        value.push_str(&s[start..pos]);

        if pos >= b.len() {
            break;
        }

        let quote_or_backslash = b[pos];
        pos += 1;
        if quote_or_backslash == b'\\' {
            match s[pos..].chars().next() {
                Some(c) => {
                    value.push(c);
                    pos += c.len_utf8();
                }
                None => {
                    value.push('\\');
                    break;
                }
            }
        } else {
            break;
        }
    }

    (value, pos)
}

pub(crate) fn serialize_parameters(output: &mut String, parameters: &[(String, String)]) {
    for (name, value) in parameters {
        output.push(';');
        output.push_str(name);
        output.push('=');
        if !value.is_empty() && value.chars().all(is_token_char) {
            output.push_str(value);
        } else {
            output.push('"');
            for c in value.chars() {
                if c == '"' || c == '\\' {
                    output.push('\\');
                }
                output.push(c);
            }
            output.push('"');
        }
    }
}

#[cfg(test)]
#[path = "../tests/common/json.rs"]
mod json;

#[cfg(test)]
mod tests {
    use super::json::{self, Json};
    use super::*;
    use encoding_rs::Encoding;

    fn run_wpt_mime_type_tests(fixture: &str) {
        let mut count = 0;

        for test in json::parse(fixture).as_array() {
            // Strings in between test objects are section titles
            if let Json::Object(_) = test {
                let input = test.get("input").as_str();
                let expected = test.get("output");

                match parse(input) {
                    Some(mime_type) => {
                        let mut serialized = mime_type.essence.clone();
                        serialize_parameters(&mut serialized, &mime_type.parameters);
                        assert_eq!(
                            Some(serialized.as_str()),
                            expected.as_opt_str(),
                            "{input:?}"
                        );

                        // The "encoding" fields aren't in every fixture
                        if let Some(expected_encoding) = test.get_opt("encoding") {
                            let encoding = mime_type
                                .parameters
                                .iter()
                                .find(|(name, _)| name == "charset")
                                .and_then(|(_, value)| Encoding::for_label(value.as_bytes()))
                                .map(|e| e.name());
                            assert_eq!(encoding, expected_encoding.as_opt_str(), "{input:?}");
                        }
                    }
                    None => assert!(expected.is_null(), "{input:?} must not be parsed"),
                }

                count += 1;
            }
        }

        assert!(count > 0);
    }

    #[test]
    fn wpt_mime_types() {
        run_wpt_mime_type_tests(include_str!("../tests/wpt/mime-types.json"));
    }

    #[test]
    fn wpt_generated_mime_types() {
        run_wpt_mime_type_tests(include_str!("../tests/wpt/generated-mime-types.json"));
    }
}
