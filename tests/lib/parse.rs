//  ██████╗  █████╗ ███████╗███████╗██╗███╗   ██╗ ██████╗
//  ██╔══██╗██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝
//  ██████╔╝███████║███████╗███████╗██║██╔██╗ ██║██║  ███╗
//  ██╔═══╝ ██╔══██║╚════██║╚════██║██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║███████║███████║██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝

#[cfg(test)]
mod passing {
    use dataurl::{DataUrl, DataUrlParseError};

    #[test]
    fn must_trim_spaces_around_url() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse(" data:, a b ")?;

        assert_eq!(data_url.data(), " a b".as_bytes());

        Ok(())
    }

    #[test]
    fn must_be_able_to_parse_url_with_no_media_type() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:,Hello,%20World!")?;

        assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello, World!");

        Ok(())
    }

    #[test]
    fn must_parse_query_as_part_of_data() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;,Hello?World#")?;

        assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello?World");

        Ok(())
    }

    #[test]
    fn must_parse_empty_query_as_part_of_data() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;,Hello?#")?;

        assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello?");

        Ok(())
    }

    #[test]
    fn must_parse_utf8_charset_no_media_type() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=utf8,")?;

        assert_eq!(data_url.charset(), "UTF-8".to_string());

        Ok(())
    }

    #[test]
    fn must_parse_utf8_charset_no_media_type_encoded() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=utf8;base64,")?;

        assert_eq!(data_url.charset(), "UTF-8".to_string());

        Ok(())
    }

    #[test]
    fn must_parse_utf8_charset_with_media_type() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:text/html;charset=utf8,")?;

        assert_eq!(data_url.charset(), "UTF-8".to_string());

        Ok(())
    }

    #[test]
    fn must_parse_utf8_charset_with_media_type_encoded() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:text/html;charset=utf8;base64,")?;

        assert_eq!(data_url.charset(), "UTF-8".to_string());

        Ok(())
    }

    #[test]
    fn must_parse_unicode_emoji() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=utf8;base64,4piA77iP")?;

        assert_eq!(data_url.data(), [226, 152, 128, 239, 184, 143]);

        Ok(())
    }

    #[test]
    fn must_parse_all_caps() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:TEXT/CSS;CHARSET=UTF8;BASE64,w5w=")?;

        assert_eq!(data_url.media_type(), "text/css".to_string());
        assert_eq!(data_url.charset(), "UTF-8".to_string());
        assert!(data_url.base64_encoded());
        assert_eq!(data_url.data(), [195, 156]);
        assert_eq!(data_url.text(), "Ü");
        assert_eq!(data_url.fragment(), None);

        Ok(())
    }

    #[test]
    fn must_parse_unicode_emoji_if_has_extra_meta_data() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl =
            DataUrl::parse("data:TEXT/CSS;filename=x.css;charset=Utf-8;BASE64;somethingelse,w5w=")?;

        assert_eq!(data_url.media_type(), "text/css".to_string());
        assert_eq!(data_url.charset(), "UTF-8".to_string());
        assert!(data_url.base64_encoded());
        assert_eq!(data_url.data(), [195, 156]);
        assert_eq!(data_url.text(), "Ü");
        assert_eq!(data_url.fragment(), None);

        Ok(())
    }

    #[test]
    fn must_parse_unicode_emoji_if_has_extra_meta_data_and_multiple_charsets(
    ) -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse(
            "data:TEXT/CSS;charset=Utf-8;filename=x;charset=US-ASCII;BASE64;somethingelse,w5w=",
        )?;

        assert_eq!(data_url.media_type(), "text/css".to_string());
        assert_eq!(data_url.charset(), "UTF-8".to_string());
        assert!(data_url.base64_encoded());
        assert_eq!(data_url.data(), [195, 156]);
        assert_eq!(data_url.text(), "Ü");
        assert_eq!(data_url.fragment(), None);

        Ok(())
    }

    // #[test]
    // fn must_parse_unicode_emoji_if_has_extra_meta_data_and_multiple_charsets_with_spaces(
    // ) -> Result<(), DataUrlParseError> {
    //     let data_url: DataUrl = DataUrl::parse(
    //         "data:TEXT/CSS; charset=Utf-8; filename=x; charset = US-ASCII; BASE64; somethingelse ,w5w=",
    //     )?;

    //     assert_eq!(data_url.media_type(), "text/css".to_string());
    //     assert_eq!(data_url.charset(), "UTF-8".to_string());
    //     assert!(data_url.encoded());
    //     assert_eq!(data_url.data(), [195, 156]);
    //     assert_eq!(data_url.text(), "Ü");
    //     assert_eq!(data_url.fragment(), None);

    //     Ok(())
    // }
}

//  ███████╗ █████╗ ██╗██╗     ██╗███╗   ██╗ ██████╗
//  ██╔════╝██╔══██╗██║██║     ██║████╗  ██║██╔════╝
//  █████╗  ███████║██║██║     ██║██╔██╗ ██║██║  ███╗
//  ██╔══╝  ██╔══██║██║██║     ██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║██║███████╗██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝

#[cfg(test)]
mod failing {
    use dataurl::{DataUrl, DataUrlParseError};

    #[test]
    fn must_error_out_if_given_empty_string() -> Result<(), DataUrlParseError> {
        match DataUrl::parse("") {
            Ok(_data_url) => {
                assert!(false);
            }
            Err(_data_url_parse_err) => {
                assert!(true);
            }
        }

        Ok(())
    }

    #[test]
    fn must_treat_data_as_base64_unencoded_if_no_semicolon_before_base64(
    ) -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:base64,SGVsbG8sIHdvcmxkIQo=")?;

        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(!data_url.base64_encoded());
        assert_eq!(
            String::from_utf8_lossy(data_url.data()),
            "SGVsbG8sIHdvcmxkIQo="
        );
        assert_eq!(data_url.fragment(), None);

        Ok(())
    }

    #[test]
    fn must_fall_back_to_us_ascii_if_given_bad_charset() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=BAD-CHARSET;base64,")?;

        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(data_url.base64_encoded());
        assert_eq!(data_url.data(), []);
        assert_eq!(data_url.fragment(), None);

        Ok(())
    }

    #[test]
    fn must_fall_back_to_text_plain_if_given_bad_media_type() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:bad;,")?;

        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(!data_url.base64_encoded());
        assert_eq!(data_url.data(), []);
        assert_eq!(data_url.fragment(), None);

        Ok(())
    }

    #[test]
    fn must_fall_to_parse_charset_if_put_not_right_after_media_type(
    ) -> Result<(), DataUrlParseError> {
        let mut data_url: DataUrl = DataUrl::parse("data:text/css;base64;charset=utf8,w5w=")?;

        assert_eq!(data_url.media_type(), "text/css".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(data_url.base64_encoded());
        assert_eq!(data_url.data(), [195, 156]);
        assert_eq!(data_url.fragment(), None);
        assert_eq!(data_url.text(), "Ãœ");
        // Different from the original her because we needed to encode "Ãœ" as US-ASCII, which is 4 bytes
        assert_eq!(data_url.to_string(), "data:text/css;base64,w4PFkw==");
        data_url.set_charset(Some("utf-8".to_string()));
        // And now it should be fine
        assert_eq!(
            data_url.to_string(),
            "data:text/css;charset=UTF-8;base64,w5w="
        );

        Ok(())
    }
}
