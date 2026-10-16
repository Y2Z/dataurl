use dataurl::{DataUrl, Error};

const LIZARD_PNG: &[u8] = include_bytes!("_data_/lizard.png");

mod parse {
    use super::*;

    #[test]
    fn empty() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:,")?;
        assert_eq!(data_url.media_type(), "text/plain");
        assert_eq!(data_url.media_type_no_default(), None);
        assert_eq!(data_url.mime_type(), "text/plain;charset=US-ASCII");
        assert_eq!(data_url.charset(), "US-ASCII");
        assert_eq!(data_url.charset_no_default(), None);
        assert!(!data_url.is_base64_encoded());
        assert_eq!(data_url.data(), b"");
        assert_eq!(data_url.fragment(), None);
        Ok(())
    }

    #[test]
    fn base64_with_plus_and_slash() -> Result<(), Error> {
        // The standard base64 alphabet uses "+" and "/" (0.1.x expected "-" and "_" instead)
        let data_url = DataUrl::parse("data:application/octet-stream;base64,+/8=")?;
        assert_eq!(data_url.data(), [0xFB, 0xFF]);
        assert!(data_url.is_base64_encoded());
        Ok(())
    }

    #[test]
    fn base64_without_padding_and_with_whitespace() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:;base64,SGVs bG8\n")?;
        assert_eq!(data_url.data(), b"Hello");
        Ok(())
    }

    #[test]
    fn ignores_tabs_and_newlines_anywhere() -> Result<(), Error> {
        let data_url = DataUrl::parse("  data:image/png;\nbase64,\n\tiVBO\r\nRw==\n")?;
        assert_eq!(data_url.media_type(), "image/png");
        assert_eq!(data_url.data(), [0x89, 0x50, 0x4E, 0x47]);
        Ok(())
    }

    #[test]
    fn scheme_and_media_type_are_case_insensitive() -> Result<(), Error> {
        let data_url = DataUrl::parse("DATA:Text/HTML;CharSet=UTF-8,x")?;
        assert_eq!(data_url.media_type(), "text/html");
        assert_eq!(data_url.charset(), "UTF-8");
        Ok(())
    }

    #[test]
    fn charset_after_whitespace() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:text/html; charset=utf-8,x")?;
        assert_eq!(data_url.charset(), "utf-8");
        assert_eq!(data_url.mime_type(), "text/html;charset=utf-8");
        Ok(())
    }

    #[test]
    fn quoted_charset() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:text/plain;charset=\"utf-8\",x")?;
        assert_eq!(data_url.charset(), "utf-8");
        Ok(())
    }

    #[test]
    fn keeps_other_parameters() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:text/plain;format=flowed;charset=utf-8,x")?;
        assert_eq!(
            data_url.parameters().collect::<Vec<_>>(),
            [("format", "flowed"), ("charset", "utf-8")]
        );
        assert_eq!(data_url.parameter("FORMAT"), Some("flowed"));
        Ok(())
    }

    #[test]
    fn many_parameters() -> Result<(), Error> {
        // 0.1.x used an i8 counter here, which overflowed
        let input = format!("data:text/plain{},x", ";a".repeat(300));
        assert_eq!(DataUrl::parse(&input)?.media_type(), "text/plain");
        Ok(())
    }

    #[test]
    fn percent_encoded_binary_data() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:image/png,%89PNG%0D%0A%1A%0A")?;
        assert_eq!(data_url.data(), b"\x89PNG\r\n\x1a\n");
        Ok(())
    }

    #[test]
    fn non_ascii_data_becomes_utf8() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:,Привет")?;
        assert_eq!(data_url.data(), "Привет".as_bytes());
        Ok(())
    }

    #[test]
    fn query_is_part_of_data() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:text/html,text<a id%3D\"b\">ok</a>?a=v#f")?;
        assert_eq!(data_url.data(), b"text<a id=\"b\">ok</a>?a=v");
        assert_eq!(data_url.fragment(), Some("f"));
        Ok(())
    }

    #[test]
    fn empty_fragment() -> Result<(), Error> {
        assert_eq!(DataUrl::parse("data:,x#")?.fragment(), Some(""));
        Ok(())
    }

    #[test]
    fn hierarchical_form() -> Result<(), Error> {
        assert_eq!(DataUrl::parse("data://host/a/../,X")?.data(), b"X");
        assert_eq!(DataUrl::parse("data:/x/./,X")?.data(), b"X");
        assert_eq!(DataUrl::parse("data://[::1]:80/,X")?.data(), b"X");
        assert_eq!(
            DataUrl::parse("data://host:port/,X"),
            Err(Error::InvalidUrl)
        );
        assert_eq!(DataUrl::parse("data://:80/,X"), Err(Error::InvalidUrl));
        Ok(())
    }

    #[test]
    fn errors() {
        assert_eq!(DataUrl::parse(""), Err(Error::NotADataUrl));
        assert_eq!(DataUrl::parse("http://a/,b"), Err(Error::NotADataUrl));
        assert_eq!(DataUrl::parse("dat"), Err(Error::NotADataUrl));
        assert_eq!(DataUrl::parse("data:text/html"), Err(Error::MissingComma));
        assert_eq!(
            DataUrl::parse("data:,x#a,b").map(|d| d.data().to_vec()),
            Ok(b"x".to_vec())
        );
        assert_eq!(DataUrl::parse("data:#,x"), Err(Error::MissingComma));
        assert_eq!(DataUrl::parse("data:;base64,a"), Err(Error::InvalidBase64));
        assert_eq!(
            DataUrl::parse("data:;base64,-_8="),
            Err(Error::InvalidBase64)
        );
    }

    #[test]
    fn from_str_and_try_from() -> Result<(), Error> {
        let a: DataUrl = "data:,x".parse()?;
        let b = DataUrl::try_from("data:,x")?;
        assert_eq!(a, b);
        Ok(())
    }
}

mod serialize {
    use super::*;

    #[test]
    fn empty() {
        let mut data_url = DataUrl::new();
        assert_eq!(data_url.to_string(), "data:,");
        data_url.set_is_base64_encoded(true);
        assert_eq!(data_url.to_string(), "data:;base64,");
    }

    #[test]
    fn readable_percent_encoding() {
        let mut data_url = DataUrl::new();
        data_url.set_data(b"Hello, World! 100% <ok> (\"#\")");
        assert_eq!(
            data_url.to_string(),
            "data:,Hello,%20World!%20100%25%20%3Cok%3E%20%28%22%23%22%29"
        );
    }

    #[test]
    fn base64_uses_standard_alphabet() {
        let mut data_url = DataUrl::new();
        data_url.set_media_type(Some("image/png")).unwrap();
        data_url.set_is_base64_encoded(true);
        data_url.set_data(&[0xFB, 0xFF]);
        assert_eq!(data_url.to_string(), "data:image/png;base64,+/8=");
    }

    #[test]
    fn data_is_never_transcoded() {
        // windows-1252 bytes for "café", which 0.1.x mangled into U+FFFD
        let mut data_url = DataUrl::new();
        data_url.set_charset(Some("windows-1252")).unwrap();
        data_url.set_data(b"caf\xE9");
        assert_eq!(data_url.to_string(), "data:;charset=windows-1252,caf%E9");
        assert_eq!(data_url.text(), "café");
    }

    #[test]
    fn parameters_without_media_type() {
        let mut data_url = DataUrl::new();
        data_url.set_charset(Some("utf8")).unwrap();
        assert_eq!(data_url.to_string(), "data:;charset=UTF-8,");
        assert_eq!(data_url.mime_type(), "text/plain;charset=UTF-8");
    }

    #[test]
    fn quoted_parameter_values() {
        let mut data_url = DataUrl::new();
        data_url.set_media_type(Some("text/plain")).unwrap();
        data_url.set_parameter("name", Some("a \"b\";c")).unwrap();
        let serialized = data_url.to_string();
        assert_eq!(serialized, "data:text/plain;name=\"a \\\"b\\\";c\",");
        assert_eq!(
            DataUrl::parse(&serialized).unwrap().parameter("name"),
            Some("a \"b\";c")
        );
    }

    #[test]
    fn fragment() {
        let mut data_url = DataUrl::new();
        data_url.set_fragment(Some("some thing"));
        assert_eq!(data_url.to_string(), "data:,#some%20thing");
        data_url.set_fragment(Some(""));
        assert_eq!(data_url.to_string(), "data:,#");
        data_url.set_fragment(None);
        assert_eq!(data_url.to_string(), "data:,");
    }

    #[test]
    fn round_trips_binary_file() -> Result<(), Error> {
        for base64 in [true, false] {
            let mut data_url = DataUrl::new();
            data_url.set_media_type(Some("image/png"))?;
            data_url.set_is_base64_encoded(base64);
            data_url.set_data(LIZARD_PNG);

            let parsed = DataUrl::parse(&data_url.to_string())?;
            assert_eq!(parsed, data_url);
            assert_eq!(parsed.into_data(), LIZARD_PNG);
        }
        Ok(())
    }
}

mod setters {
    use super::*;

    #[test]
    fn media_type() {
        let mut data_url = DataUrl::new();
        assert_eq!(data_url.set_media_type(Some(" Image/SVG+XML ")), Ok(()));
        assert_eq!(data_url.media_type(), "image/svg+xml");
        assert_eq!(data_url.set_media_type(None), Ok(()));
        assert_eq!(data_url.media_type_no_default(), None);

        for invalid in [
            "",
            "text",
            "text/",
            "/html",
            "wrong/media/type",
            "text/html;charset=utf-8",
            "te xt/html",
        ] {
            assert_eq!(
                data_url.set_media_type(Some(invalid)),
                Err(Error::InvalidMediaType),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn media_type_keeps_charset() {
        let mut data_url = DataUrl::new();
        data_url.set_charset(Some("utf-8")).unwrap();
        data_url.set_media_type(Some("text/html")).unwrap();
        assert_eq!(data_url.mime_type(), "text/html;charset=UTF-8");
    }

    #[test]
    fn charset() {
        let mut data_url = DataUrl::new();
        assert_eq!(data_url.set_charset(Some("utf8")), Ok(()));
        assert_eq!(data_url.charset(), "UTF-8");
        assert_eq!(data_url.set_charset(Some("gb2312")), Ok(()));
        assert_eq!(data_url.charset(), "GBK");
        assert_eq!(data_url.parameters().count(), 1);
        assert_eq!(
            data_url.set_charset(Some("BAD-CHARSET")),
            Err(Error::InvalidCharset)
        );
        assert_eq!(
            data_url.set_charset(Some("iso-2022-kr")),
            Err(Error::InvalidCharset)
        );
        assert_eq!(data_url.charset(), "GBK");
        assert_eq!(data_url.set_charset(None), Ok(()));
        assert_eq!(data_url.charset_no_default(), None);
    }

    #[test]
    fn parameter() {
        let mut data_url = DataUrl::new();
        assert_eq!(data_url.set_parameter("Name", Some("value")), Ok(()));
        assert_eq!(data_url.parameter("name"), Some("value"));
        assert_eq!(data_url.set_parameter("name", Some("other")), Ok(()));
        assert_eq!(
            data_url.parameters().collect::<Vec<_>>(),
            [("name", "other")]
        );
        assert_eq!(data_url.set_parameter("name", None), Ok(()));
        assert_eq!(data_url.parameters().count(), 0);

        assert_eq!(
            data_url.set_parameter("", Some("x")),
            Err(Error::InvalidParameter)
        );
        assert_eq!(
            data_url.set_parameter("a b", Some("x")),
            Err(Error::InvalidParameter)
        );
        for invalid in ["a,b", "a#b", "é", "\n"] {
            assert_eq!(
                data_url.set_parameter("x", Some(invalid)),
                Err(Error::InvalidParameter)
            );
        }
    }

    #[test]
    fn text() -> Result<(), Error> {
        let mut data_url = DataUrl::new();

        data_url.set_text("café")?; // Default (US-ASCII, i.e. windows-1252)
        assert_eq!(data_url.data(), b"caf\xE9");
        assert_eq!(data_url.set_text("☀"), Err(Error::UnencodableText));
        assert_eq!(data_url.data(), b"caf\xE9");

        data_url.set_charset(Some("UTF-8"))?;
        data_url.set_text("☀")?;
        assert_eq!(data_url.data(), "☀".as_bytes());

        data_url.set_charset(Some("utf-16le"))?;
        data_url.set_text("A☀")?;
        assert_eq!(data_url.data(), [0x41, 0x00, 0x00, 0x26]);
        assert_eq!(data_url.text(), "A☀");

        data_url.set_charset(Some("gbk"))?;
        data_url.set_text("你好")?;
        assert_eq!(data_url.data(), [0xC4, 0xE3, 0xBA, 0xC3]);
        assert_eq!(data_url.text(), "你好");
        Ok(())
    }
}

mod misc {
    use super::*;

    #[test]
    fn decodes_text_in_declared_charset() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:;charset=gbk;base64,PbnjtqvKocnu29rK0LGmsLI=")?;
        assert_eq!(data_url.text(), "=广东省深圳市宝安");
        assert_eq!(data_url.encoding(), encoding_rs::GBK);
        Ok(())
    }

    #[test]
    fn unknown_charset_decodes_as_windows_1252() -> Result<(), Error> {
        let data_url = DataUrl::parse("data:;charset=whatever,%E9")?;
        assert_eq!(data_url.charset(), "whatever");
        assert_eq!(data_url.text(), "é");
        Ok(())
    }

    #[test]
    fn is_binary() -> Result<(), Error> {
        for (input, binary) in [
            ("data:,", false),
            ("data:text/html,", false),
            ("data:application/json,", false),
            ("data:image/svg+xml,", false),
            ("data:application/ld+json,", false),
            ("data:image/png,", true),
            ("data:application/octet-stream,", true),
        ] {
            assert_eq!(DataUrl::parse(input)?.is_binary(), binary, "{input}");
        }
        Ok(())
    }

    #[test]
    fn error_is_a_std_error() {
        let result: Result<DataUrl, Box<dyn std::error::Error>> =
            "data:;base64,a".parse::<DataUrl>().map_err(Into::into);
        assert_eq!(
            result.unwrap_err().to_string(),
            "data URL contains invalid base64"
        );
    }

    #[test]
    fn debug_does_not_dump_data() {
        let mut data_url = DataUrl::new();
        data_url.set_data(LIZARD_PNG);
        let debug = format!("{data_url:?}");
        assert!(debug.contains("[14196 bytes]"), "{debug}");
        assert!(debug.len() < 200);
    }
}
