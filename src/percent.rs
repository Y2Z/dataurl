//! Percent-encoding (<https://url.spec.whatwg.org/#percent-encoded-bytes>).

const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Bytes that get percent-encoded in the data portion of a data URL.
/// Besides what URLs require (controls, space, non-ASCII, "%", "#"), this also covers
/// characters that would break out of HTML attributes or unquoted CSS url() values.
pub(crate) fn in_data_set(byte: u8) -> bool {
    !matches!(byte, 0x21..=0x7E)
        || matches!(
            byte,
            b'"' | b'#' | b'%' | b'\'' | b'(' | b')' | b'<' | b'>' | b'\\' | b'`'
        )
}

/// Bytes that get percent-encoded in a fragment ("%" is kept as is, to preserve existing escapes).
pub(crate) fn in_fragment_set(byte: u8) -> bool {
    !matches!(byte, 0x21..=0x7E) || matches!(byte, b'"' | b'<' | b'>' | b'`')
}

/// Encoding applied to the metadata part when parsing, to mirror what URL parsers do
/// to opaque paths: C0 controls and non-ASCII get escaped, everything else stays.
pub(crate) fn in_c0_control_set(byte: u8) -> bool {
    !(0x20..=0x7E).contains(&byte)
}

pub(crate) fn encode_into(output: &mut String, input: &[u8], should_encode: fn(u8) -> bool) {
    output.reserve(input.len());

    for &byte in input {
        if should_encode(byte) {
            output.push('%');
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0F) as usize] as char);
        } else {
            output.push(byte as char);
        }
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|d| d as u8)
}

/// Malformed sequences (a "%" not followed by two hex digits) are left as they are.
pub(crate) fn decode(input: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        if input[i] == b'%' && i + 2 < input.len() {
            if let (Some(high), Some(low)) = (hex_value(input[i + 1]), hex_value(input[i + 2])) {
                output.push(high << 4 | low);
                i += 3;
                continue;
            }
        }
        output.push(input[i]);
        i += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_valid_and_keeps_malformed_sequences() {
        assert_eq!(decode(b"%41%4a%zz%4%"), b"AJ%zz%4%");
        assert_eq!(decode(b"%FF"), [0xFF]);
    }

    #[test]
    fn encoding_round_trips() {
        let data: Vec<u8> = (0..=255).collect();
        let mut s = String::new();
        encode_into(&mut s, &data, in_data_set);
        assert!(s.is_ascii());
        assert_eq!(decode(s.as_bytes()), data);
    }
}
