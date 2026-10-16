use dataurl::{DataEncoding, DataUrl, Error, Escaping};
use std::io;

const LIZARD_PNG: &[u8] = include_bytes!("_data_/lizard.png");

/// Deterministic pseudo-random bytes (xorshift), so tests don't need a dependency for randomness
fn pseudo_random_bytes(length: usize, mut seed: u64) -> Vec<u8> {
    (0..length)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as u8
        })
        .collect()
}

fn all_settings() -> Vec<(DataEncoding, Escaping)> {
    let mut settings = Vec::new();
    for data_encoding in [
        DataEncoding::Percent,
        DataEncoding::Base64,
        DataEncoding::Shortest,
    ] {
        for escaping in [Escaping::Full, Escaping::Minimal] {
            settings.push((data_encoding, escaping));
        }
    }
    settings
}

#[test]
fn set_data_accepts_owned_and_borrowed_data() {
    let mut data_url = DataUrl::new();

    data_url.set_data(vec![1, 2, 3]);
    assert_eq!(data_url.data(), [1, 2, 3]);
    data_url.set_data(&[4, 5][..]);
    assert_eq!(data_url.data(), [4, 5]);
    data_url.set_data([6, 7, 8]);
    assert_eq!(data_url.data(), [6, 7, 8]);
    data_url.set_data("text");
    assert_eq!(data_url.data(), b"text");
    data_url.set_data(String::from("owned text"));
    assert_eq!(data_url.data(), b"owned text");
}

#[test]
fn set_data_moves_vectors_without_copying() {
    let data = pseudo_random_bytes(1024, 1);
    let pointer = data.as_ptr();

    let mut data_url = DataUrl::new();
    data_url.set_data(data);
    assert_eq!(data_url.data().as_ptr(), pointer);
    let data = data_url.into_data();
    assert_eq!(data.as_ptr(), pointer);
}

#[test]
fn encoded_len_is_exact() -> Result<(), Error> {
    let datas: Vec<Vec<u8>> = vec![
        vec![],
        b" ".to_vec(),
        b"trailing  ".to_vec(),
        LIZARD_PNG.to_vec(),
        (0..=255).collect(),
        pseudo_random_bytes(10_000, 2),
    ];

    for data in &datas {
        for (data_encoding, escaping) in all_settings() {
            for fragment in [None, Some(""), Some("some fragment")] {
                let mut data_url = DataUrl::new();
                data_url.set_media_type(Some("image/svg+xml"))?;
                data_url.set_parameter("name", Some("a \"b\""))?;
                data_url.set_data_encoding(data_encoding);
                data_url.set_escaping(escaping);
                data_url.set_fragment(fragment);
                data_url.set_data(data.as_slice());

                assert_eq!(
                    data_url.encoded_len(),
                    data_url.to_string().len(),
                    "{data_encoding:?} {escaping:?} {fragment:?}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn write_to_matches_display() -> Result<(), Error> {
    for (data_encoding, escaping) in all_settings() {
        let mut data_url = DataUrl::new();
        data_url.set_charset(Some("utf-8"))?;
        data_url.set_data_encoding(data_encoding);
        data_url.set_escaping(escaping);
        data_url.set_fragment(Some("frag"));
        data_url.set_data(pseudo_random_bytes(100_000, 3));

        let mut written: Vec<u8> = Vec::new();
        data_url.write_to(&mut written).unwrap();
        assert_eq!(String::from_utf8(written).unwrap(), data_url.to_string());
    }
    Ok(())
}

#[test]
fn write_to_reports_io_errors() {
    struct FailingWriter;

    impl io::Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut data_url = DataUrl::new();
    data_url.set_data("x");
    let error = data_url.write_to(FailingWriter).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn large_data_round_trips() -> Result<(), Error> {
    // Big enough to span many of the serializer's chunks
    let data = pseudo_random_bytes(1024 * 1024 + 1, 4);

    for (data_encoding, escaping) in all_settings() {
        let mut data_url = DataUrl::new();
        data_url.set_media_type(Some("application/octet-stream"))?;
        data_url.set_data_encoding(data_encoding);
        data_url.set_escaping(escaping);
        data_url.set_data(data.as_slice());

        let serialized = data_url.to_string();
        assert_eq!(serialized.len(), data_url.encoded_len());
        assert_eq!(DataUrl::parse(&serialized)?.into_data(), data);
    }
    Ok(())
}

#[test]
fn base64_with_and_without_percent_encoded_characters() -> Result<(), Error> {
    // Both decoding paths: straight from the input, and after percent-decoding
    assert_eq!(DataUrl::parse("data:;base64,SGVsbG8=")?.data(), b"Hello");
    assert_eq!(DataUrl::parse("data:;base64,SGVsbG8%3D")?.data(), b"Hello");
    assert_eq!(DataUrl::parse("data:;base64,SGVs%20bG8")?.data(), b"Hello");
    assert_eq!(
        DataUrl::parse("data:;base64,%ZZ"),
        Err(Error::InvalidBase64),
        "malformed escapes are kept as is, and aren't valid base64"
    );
    Ok(())
}
