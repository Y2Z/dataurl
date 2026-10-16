//! Standard base64 (RFC 4648 §4, the alphabet with "+" and "/"), which is what data URLs use.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Writes base64 in chunks, so that large data doesn't need one big intermediate string.
pub(crate) fn write<W: std::fmt::Write>(writer: &mut W, data: &[u8]) -> std::fmt::Result {
    // A multiple of 3, so that only the very last chunk can need padding
    const CHUNK_SIZE: usize = 6 * 1024;
    let mut buffer = String::with_capacity(CHUNK_SIZE / 3 * 4);

    for chunk in data.chunks(CHUNK_SIZE) {
        buffer.clear();
        encode_into(&mut buffer, chunk);
        writer.write_str(&buffer)?;
    }

    Ok(())
}

/// Length of the base64 encoding of `length` bytes, padding included.
pub(crate) fn encoded_length(length: usize) -> usize {
    length.div_ceil(3) * 4
}

pub(crate) fn encode_into(output: &mut String, data: &[u8]) {
    output.reserve(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;

        output.push(ALPHABET[(n >> 18) as usize & 63] as char);
        output.push(ALPHABET[(n >> 12) as usize & 63] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
}

fn decode_char(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some((c - b'A') as u32),
        b'a'..=b'z' => Some((c - b'a' + 26) as u32),
        b'0'..=b'9' => Some((c - b'0' + 52) as u32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// The "forgiving-base64 decode" algorithm from the WHATWG Infra Standard
/// (<https://infra.spec.whatwg.org/#forgiving-base64-decode>): ASCII whitespace is ignored,
/// padding is optional, and anything else outside the alphabet makes decoding fail.
pub(crate) fn decode_forgiving(input: &[u8]) -> Option<Vec<u8>> {
    let is_not_whitespace = |c: &&u8| !matches!(c, b'\t' | b'\n' | 0x0C | b'\r' | b' ');

    // Work out the length without whitespace and padding first, instead of making a stripped copy
    let mut length = input.iter().filter(is_not_whitespace).count();
    if length % 4 == 0 {
        let mut from_the_end = input.iter().rev().filter(is_not_whitespace);
        if from_the_end.next() == Some(&b'=') {
            length -= 1;
            if from_the_end.next() == Some(&b'=') {
                length -= 1;
            }
        }
    }

    if length % 4 == 1 {
        return None;
    }

    let mut output: Vec<u8> = Vec::with_capacity(length / 4 * 3 + 2);
    let mut buffer: u32 = 0;
    let mut bits: u8 = 0;

    // The padding that was accounted for comes last, so stopping after `length` characters skips it
    for &c in input.iter().filter(is_not_whitespace).take(length) {
        buffer = buffer << 6 | decode_char(c)?;
        bits += 6;
        if bits == 24 {
            output.extend_from_slice(&[(buffer >> 16) as u8, (buffer >> 8) as u8, buffer as u8]);
            buffer = 0;
            bits = 0;
        }
    }

    // Leftover bits that don't form a whole byte get discarded
    match bits {
        12 => output.push((buffer >> 4) as u8),
        18 => output.extend_from_slice(&[(buffer >> 10) as u8, (buffer >> 2) as u8]),
        _ => {}
    }

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_with_standard_alphabet_and_padding() {
        let mut s = String::new();
        encode_into(&mut s, &[0xFB, 0xFF]);
        assert_eq!(s, "+/8=");

        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            let mut s = String::new();
            encode_into(&mut s, input.as_bytes());
            assert_eq!(s, expected);
            assert_eq!(decode_forgiving(s.as_bytes()).unwrap(), input.as_bytes());
        }
    }

    #[test]
    fn round_trips_all_byte_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1000).collect();
        let mut s = String::new();
        encode_into(&mut s, &data);
        assert_eq!(decode_forgiving(s.as_bytes()).unwrap(), data);
    }

    #[test]
    fn chunked_writer_matches_one_go_encoding() {
        let data: Vec<u8> = (0..=255).cycle().take(6 * 1024 * 3 + 7).collect();
        let mut chunked = String::new();
        write(&mut chunked, &data).unwrap();
        let mut whole = String::new();
        encode_into(&mut whole, &data);
        assert_eq!(chunked, whole);
        assert_eq!(chunked.len(), encoded_length(data.len()));
    }

    #[test]
    fn rejects_url_safe_alphabet() {
        assert_eq!(decode_forgiving(b"-_8="), None);
    }
}
