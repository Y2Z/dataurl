use dataurl::{DataEncoding, DataUrl, Error, Escaping};

const LIZARD_PNG: &[u8] = include_bytes!("_data_/lizard.png");

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path fill="#09f" d="M8 0a8 8 0 1 0 0 16A8 8 0 0 0 8 0z"/><text x="2" y="12">A &amp; B's</text></svg>"##;

fn serialize(data: &[u8], data_encoding: DataEncoding, escaping: Escaping) -> String {
    let mut data_url = DataUrl::new();
    data_url.set_data_encoding(data_encoding);
    data_url.set_escaping(escaping);
    data_url.set_data(data);
    data_url.to_string()
}

#[test]
fn defaults() {
    let data_url = DataUrl::new();
    assert_eq!(data_url.data_encoding(), DataEncoding::Percent);
    assert_eq!(data_url.escaping(), Escaping::Full);
}

#[test]
fn parsed_encoding() -> Result<(), Error> {
    assert_eq!(
        DataUrl::parse("data:,x")?.data_encoding(),
        DataEncoding::Percent
    );
    assert_eq!(
        DataUrl::parse("data:;base64,eA==")?.data_encoding(),
        DataEncoding::Base64
    );
    Ok(())
}

#[test]
fn full_escaping() {
    assert_eq!(
        serialize(b"a b'(c)`\"<&>\\#%", DataEncoding::Percent, Escaping::Full),
        "data:,a%20b%27%28c%29%60%22%3C%26%3E%5C%23%25"
    );
}

#[test]
fn minimal_escaping() {
    assert_eq!(
        serialize(
            b"a b'(c)`\"<&>\\#%",
            DataEncoding::Percent,
            Escaping::Minimal
        ),
        "data:,a b'(c)`%22%3C%26%3E%5C%23%25"
    );
    // Controls, non-ASCII and trailing spaces always get escaped
    assert_eq!(
        serialize(
            "\tcafé \n  ".as_bytes(),
            DataEncoding::Percent,
            Escaping::Minimal
        ),
        "data:,%09caf%C3%A9 %0A%20%20"
    );
}

#[test]
fn minimal_escaping_is_safe_in_double_quotes_and_style_elements() {
    let serialized = serialize(SVG, DataEncoding::Percent, Escaping::Minimal);
    for forbidden in ['"', '\\', '<', '>', '&', '#', '\n'] {
        assert!(
            !serialized.contains(forbidden),
            "{forbidden:?} in {serialized}"
        );
    }
    assert!(serialized.is_ascii());
}

#[test]
fn shortest_picks_percent_for_text_and_base64_for_binary() {
    let text = serialize(SVG, DataEncoding::Shortest, Escaping::Minimal);
    assert!(text.starts_with("data:,%3Csvg xmlns="), "{text}");

    let binary = serialize(LIZARD_PNG, DataEncoding::Shortest, Escaping::Minimal);
    assert!(binary.starts_with("data:;base64,iVBOR"));

    // Whichever is picked, it's never longer than either option
    for data in [SVG, LIZARD_PNG, b"", b" ", b"\x00\x01\x02"] {
        for escaping in [Escaping::Full, Escaping::Minimal] {
            let shortest = serialize(data, DataEncoding::Shortest, escaping).len();
            assert!(shortest <= serialize(data, DataEncoding::Percent, escaping).len());
            assert!(shortest <= serialize(data, DataEncoding::Base64, escaping).len());
        }
    }
}

#[test]
fn shortest_prefers_percent_on_ties() {
    // "abc" is 3 characters percent-encoded, and ";base64" + "YWJj" is 11
    assert_eq!(
        serialize(b"abc", DataEncoding::Shortest, Escaping::Full),
        "data:,abc"
    );
    assert_eq!(
        serialize(b"", DataEncoding::Shortest, Escaping::Full),
        "data:,"
    );
}

#[test]
fn minimal_escaping_makes_svg_much_shorter() {
    let full = serialize(SVG, DataEncoding::Percent, Escaping::Full).len();
    let minimal = serialize(SVG, DataEncoding::Percent, Escaping::Minimal).len();
    let base64 = serialize(SVG, DataEncoding::Base64, Escaping::Full).len();
    assert!(
        minimal < base64 && base64 < full,
        "minimal {minimal}, base64 {base64}, full {full}"
    );
}

#[test]
fn every_combination_round_trips() -> Result<(), Error> {
    let all_bytes: Vec<u8> = (0..=255).collect();
    for data in [SVG, LIZARD_PNG, &all_bytes, b"  leading and trailing  "] {
        for data_encoding in [
            DataEncoding::Percent,
            DataEncoding::Base64,
            DataEncoding::Shortest,
        ] {
            for escaping in [Escaping::Full, Escaping::Minimal] {
                let mut data_url = DataUrl::new();
                data_url.set_media_type(Some("image/svg+xml"))?;
                data_url.set_data_encoding(data_encoding);
                data_url.set_escaping(escaping);
                data_url.set_fragment(Some("frag ment"));
                data_url.set_data(data);

                let parsed = DataUrl::parse(&data_url.to_string())?;
                assert_eq!(parsed.data(), data, "{data_encoding:?} {escaping:?}");
                assert_eq!(parsed.media_type(), "image/svg+xml");
                assert_eq!(parsed.fragment(), Some("frag%20ment"));
            }
        }
    }
    Ok(())
}

#[test]
fn escaping_does_not_affect_base64() {
    assert_eq!(
        serialize(&[0xFB, 0xFF], DataEncoding::Base64, Escaping::Minimal),
        serialize(&[0xFB, 0xFF], DataEncoding::Base64, Escaping::Full)
    );
}
