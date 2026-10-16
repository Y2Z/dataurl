//! Property checks shared by the randomized tests (tests/roundtrip.rs) and the fuzz targets (fuzz/).
//! Each check panics if a property doesn't hold.

#![allow(dead_code)]

use dataurl::{DataEncoding, DataUrl, Escaping};

pub const DATA_ENCODINGS: [DataEncoding; 3] = [
    DataEncoding::Percent,
    DataEncoding::Base64,
    DataEncoding::Shortest,
];
pub const ESCAPINGS: [Escaping; 2] = [Escaping::Full, Escaping::Minimal];

/// Parsing must never panic, and whatever parses must survive being serialized and parsed again,
/// with every combination of settings.
pub fn check_parse(input: &str) {
    let Ok(data_url) = DataUrl::parse(input) else {
        return;
    };

    // Data never gets bigger than the input it was decoded from
    assert!(data_url.data().len() <= input.len(), "{input:?}");

    for data_encoding in DATA_ENCODINGS {
        for escaping in ESCAPINGS {
            let mut data_url = data_url.clone();
            data_url.set_data_encoding(data_encoding);
            data_url.set_escaping(escaping);
            let serialized = data_url.to_string();

            assert!(serialized.is_ascii(), "{input:?} -> {serialized:?}");
            assert_eq!(serialized.len(), data_url.encoded_len(), "{input:?}");

            let mut reparsed = DataUrl::parse(&serialized)
                .unwrap_or_else(|e| panic!("{input:?} -> {serialized:?} doesn't parse: {e}"));
            assert_eq!(
                reparsed.data(),
                data_url.data(),
                "{input:?} -> {serialized:?}"
            );
            assert_eq!(
                reparsed.mime_type(),
                data_url.mime_type(),
                "{input:?} -> {serialized:?}"
            );

            // A second round must give exactly the same result
            reparsed.set_data_encoding(data_encoding);
            reparsed.set_escaping(escaping);
            assert_eq!(reparsed.to_string(), serialized, "{input:?}");
        }
    }
}

/// Anything that can be set must survive being serialized and parsed.
/// `choices` drives the settings, so that fuzzers can explore them.
pub fn check_serialize(data: &[u8], choices: &[u8]) {
    const MEDIA_TYPES: [Option<&str>; 5] = [
        None,
        Some("text/plain"),
        Some("image/svg+xml"),
        Some("application/octet-stream"),
        Some("x/base64"),
    ];
    let choice = |i: usize| choices.get(i).copied().unwrap_or(0) as usize;

    let mut data_url = DataUrl::new();
    data_url
        .set_media_type(MEDIA_TYPES[choice(0) % MEDIA_TYPES.len()])
        .unwrap();
    data_url.set_data_encoding(DATA_ENCODINGS[choice(1) % DATA_ENCODINGS.len()]);
    data_url.set_escaping(ESCAPINGS[choice(2) % ESCAPINGS.len()]);

    // Parameter values: any printable ASCII except "," and "#", which can't be represented
    let value: String = choices
        .iter()
        .skip(3)
        .take(choice(3) % 8)
        .map(|&b| (b' ' + b % 95) as char)
        .filter(|&c| c != ',' && c != '#')
        .collect();
    if choice(4) % 2 == 1 {
        data_url.set_parameter("p", Some(&value)).unwrap();
    }
    if choice(5) % 3 == 1 {
        data_url.set_fragment(Some(&String::from_utf8_lossy(&data[..data.len().min(16)])));
    }
    data_url.set_data(data);

    let serialized = data_url.to_string();
    assert_eq!(serialized.len(), data_url.encoded_len());

    let reparsed =
        DataUrl::parse(&serialized).unwrap_or_else(|e| panic!("{serialized:?} doesn't parse: {e}"));
    assert_eq!(reparsed.data(), data, "{serialized:?}");
    assert_eq!(reparsed.mime_type(), data_url.mime_type(), "{serialized:?}");
}
