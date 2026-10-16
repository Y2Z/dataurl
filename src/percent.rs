//! Percent-encoding (<https://url.spec.whatwg.org/#percent-encoded-bytes>).

const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Bytes that always get percent-encoded in the data portion of a data URL: what URLs require
/// (controls, non-ASCII, "%", "#"), and what would end or alter a double-quoted HTML attribute,
/// CSS string, or a STYLE or SCRIPT element the data URL is in ("\"", "&", "<", ">", "\\").
pub(crate) fn in_minimal_data_set(byte: u8) -> bool {
    !matches!(byte, 0x20..=0x7E) || matches!(byte, b'"' | b'#' | b'%' | b'&' | b'<' | b'>' | b'\\')
}

/// Bytes that get percent-encoded in the data portion of a data URL by default: on top of the
/// minimal set, characters that would break unquoted HTML attributes, unquoted CSS url() values,
/// or srcset (space, "'", "(", ")", "`").
pub(crate) fn in_data_set(byte: u8) -> bool {
    in_minimal_data_set(byte) || matches!(byte, b' ' | b'\'' | b'(' | b')' | b'`')
}

/// Where the trailing spaces of the data start; URL parsers strip them, so they always get encoded
fn trailing_spaces_start(data: &[u8]) -> usize {
    data.len() - data.iter().rev().take_while(|&&b| b == b' ').count()
}

/// Percent-encodes the data portion of a data URL, writing it out in chunks,
/// so that large data doesn't need one big intermediate string.
pub(crate) fn write_data<W: std::fmt::Write>(
    writer: &mut W,
    data: &[u8],
    should_encode: fn(u8) -> bool,
) -> std::fmt::Result {
    const CHUNK_SIZE: usize = 8 * 1024;
    let trailing_spaces_start = trailing_spaces_start(data);
    let mut buffer = String::with_capacity(CHUNK_SIZE + 3);

    for (i, &byte) in data.iter().enumerate() {
        if should_encode(byte) || i >= trailing_spaces_start {
            push_encoded(&mut buffer, byte);
        } else {
            buffer.push(byte as char);
        }
        if buffer.len() >= CHUNK_SIZE {
            writer.write_str(&buffer)?;
            buffer.clear();
        }
    }

    writer.write_str(&buffer)
}

/// Percent-encodes the data portion of a data URL.
#[cfg(test)]
pub(crate) fn encode_data_into(output: &mut String, data: &[u8], should_encode: fn(u8) -> bool) {
    write_data(output, data, should_encode).unwrap();
}

/// How long the data portion would be once percent-encoded, without encoding it.
pub(crate) fn encoded_length(data: &[u8], should_encode: fn(u8) -> bool) -> usize {
    let trailing_spaces_start = trailing_spaces_start(data);

    data.iter()
        .enumerate()
        .map(|(i, &byte)| {
            if should_encode(byte) || i >= trailing_spaces_start {
                3
            } else {
                1
            }
        })
        .sum()
}

fn push_encoded(output: &mut String, byte: u8) {
    output.push('%');
    output.push(HEX[(byte >> 4) as usize] as char);
    output.push(HEX[(byte & 0x0F) as usize] as char);
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
            push_encoded(output, byte);
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
    fn trailing_spaces_always_get_encoded() {
        let mut s = String::new();
        encode_data_into(&mut s, b"a b  ", in_minimal_data_set);
        assert_eq!(s, "a b%20%20");
        assert_eq!(encoded_length(b"a b  ", in_minimal_data_set), s.len());
    }

    #[test]
    fn chunk_boundaries_do_not_affect_output() {
        // Trailing spaces spanning a chunk boundary, and escapes landing right on it
        for length in [8 * 1024 - 1, 8 * 1024, 8 * 1024 + 1, 3 * 8 * 1024 + 5] {
            for filler in *b"a% " {
                let mut data = vec![filler; length];
                data.extend_from_slice(b"x   ");
                let mut s = String::new();
                encode_data_into(&mut s, &data, in_minimal_data_set);
                assert_eq!(s.len(), encoded_length(&data, in_minimal_data_set));
                assert_eq!(decode(s.as_bytes()), data);
                assert!(s.ends_with("x%20%20%20"));
            }
        }
    }

    #[test]
    fn decodes_valid_and_keeps_malformed_sequences() {
        assert_eq!(decode(b"%41%4a%zz%4%"), b"AJ%zz%4%");
        assert_eq!(decode(b"%FF"), [0xFF]);
    }

    #[test]
    fn encoding_round_trips() {
        let data: Vec<u8> = (0..=255).collect();
        for set in [in_data_set, in_minimal_data_set] {
            let mut s = String::new();
            encode_data_into(&mut s, &data, set);
            assert!(s.is_ascii());
            assert_eq!(s.len(), encoded_length(&data, set));
            assert_eq!(decode(s.as_bytes()), data);
        }
    }
}
