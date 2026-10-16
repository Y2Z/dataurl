use dataurl::mime::MimeType;
use dataurl::{DataUrl, Error};

#[test]
fn parse_and_accessors() -> Result<(), Error> {
    let mime_type = MimeType::parse(" Image/SVG+XML ; Charset=\"utf-8\" ; foo=bar ")?;
    assert_eq!(mime_type.essence(), "image/svg+xml");
    assert_eq!(mime_type.main_type(), "image");
    assert_eq!(mime_type.subtype(), "svg+xml");
    assert_eq!(mime_type.charset(), Some("utf-8"));
    assert_eq!(mime_type.parameter("FOO"), Some("bar"));
    assert_eq!(mime_type.parameter("missing"), None);
    assert_eq!(
        mime_type.parameters().collect::<Vec<_>>(),
        [("charset", "utf-8"), ("foo", "bar")]
    );
    Ok(())
}

#[test]
fn display_uses_quotes_only_when_needed() -> Result<(), Error> {
    let mime_type: MimeType = "text/plain;a=b;c=\"d e\";f=\"\\\"\"".parse()?;
    assert_eq!(mime_type.to_string(), "text/plain;a=b;c=\"d e\";f=\"\\\"\"");
    assert_eq!(MimeType::parse(&mime_type.to_string())?, mime_type);
    Ok(())
}

#[test]
fn invalid_parameters_are_dropped() -> Result<(), Error> {
    // Whitespace before "=", empty values, and duplicates don't count; the MIME type itself stays valid
    let mime_type = MimeType::parse("text/html;charset =x;a=;b=1;b=2")?;
    assert_eq!(mime_type.parameters().collect::<Vec<_>>(), [("b", "1")]);
    Ok(())
}

#[test]
fn invalid_mime_types() {
    for input in [
        "",
        "text",
        "text/",
        "/html",
        "te xt/html",
        "text/html/x",
        "†/†",
    ] {
        assert_eq!(
            MimeType::parse(input),
            Err(Error::InvalidMediaType),
            "{input:?}"
        );
        assert!(input.parse::<MimeType>().is_err());
        assert!(MimeType::try_from(input).is_err());
    }
}

#[test]
fn data_url_mime_type() -> Result<(), Error> {
    let data_url = DataUrl::parse("data:,")?;
    assert_eq!(
        data_url.mime_type().to_string(),
        "text/plain;charset=US-ASCII"
    );
    assert_eq!(data_url.mime_type().charset(), Some("US-ASCII"));

    let data_url = DataUrl::parse("data:image/svg+xml;charset=utf-8,x")?;
    assert_eq!(
        data_url.mime_type(),
        MimeType::parse("image/svg+xml;charset=utf-8")?
    );
    Ok(())
}

#[test]
fn set_mime_type_from_content_type_header() -> Result<(), Error> {
    let mut data_url = DataUrl::new();
    data_url.set_parameter("old", Some("gone"))?;
    data_url.set_mime_type(Some(&MimeType::parse("Text/CSS; charset=\"UTF-8\"")?))?;
    assert_eq!(data_url.media_type(), "text/css");
    assert_eq!(data_url.charset(), "UTF-8");
    assert_eq!(data_url.parameter("old"), None);
    assert_eq!(data_url.to_string(), "data:text/css;charset=UTF-8,");

    data_url.set_mime_type(None)?;
    assert_eq!(data_url.media_type_no_default(), None);
    assert_eq!(data_url.parameters().count(), 0);
    Ok(())
}

#[test]
fn set_mime_type_rejects_unrepresentable_parameters() -> Result<(), Error> {
    // Commas end a data URL's metadata, and non-ASCII would get percent-encoded by URL parsers
    let mut data_url = DataUrl::new();
    data_url.set_media_type(Some("text/plain"))?;
    for input in ["text/html;a=\"x,y\"", "text/html;a=caf\u{e9}"] {
        let mime_type = MimeType::parse(input)?;
        assert_eq!(
            data_url.set_mime_type(Some(&mime_type)),
            Err(Error::InvalidParameter)
        );
        assert_eq!(
            data_url.media_type(),
            "text/plain",
            "nothing changes on error"
        );
    }
    Ok(())
}
