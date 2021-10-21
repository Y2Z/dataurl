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
    fn must_be_empty_by_default() -> Result<(), DataUrlParseError> {
        let data_url = DataUrl::new();

        assert_eq!(data_url.to_string(), "data:,");

        Ok(())
    }

    #[test]
    fn must_be_empty_with_base64_if_set_encoded_to_true() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_is_base64_encoded(true);

        assert_eq!(data_url.to_string(), "data:;base64,");

        Ok(())
    }

    #[test]
    fn must_be_empty_with_proper_media_type_and_base64_if_encoded_but_no_data(
    ) -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("text/html".to_string()));
        data_url.set_is_base64_encoded(true);

        assert_eq!(data_url.to_string(), "data:text/html;base64,");

        Ok(())
    }

    #[test]
    fn must_not_reflect_default_charset_even_if_it_was_set() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("text/html".to_string()));
        data_url.set_charset(Some("US-ASCII".to_string()));
        data_url.set_is_base64_encoded(true);

        assert_eq!(data_url.to_string(), "data:text/html;base64,");

        Ok(())
    }

    #[test]
    fn must_reflect_non_default_charset_if_provided() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("text/html".to_string()));
        data_url.set_charset(Some("utf8".to_string()));
        data_url.set_is_base64_encoded(true);

        assert_eq!(data_url.to_string(), "data:text/html;charset=UTF-8;base64,");

        Ok(())
    }

    #[test]
    fn must_properly_encode_utf8_emoji_in_base64() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("text/html".to_string()));
        data_url.set_charset(Some("utf8".to_string()));
        data_url.set_is_base64_encoded(true);
        data_url.set_data(&[0xe2, 0x98, 0x80, 0xef, 0xb8, 0x8f]); // Sun emoji in UTF-8 bytes

        assert_eq!(
            data_url.to_string(),
            "data:text/html;charset=UTF-8;base64,4piA77iP"
        );

        Ok(())
    }

    #[test]
    fn must_properly_encode_utf8_emoji() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("text/html".to_string()));
        data_url.set_charset(Some("utf8".to_string()));
        data_url.set_data(&[0xe2, 0x98, 0x80, 0xef, 0xb8, 0x8f]); // Sun emoji in UTF-8 bytes

        assert_eq!(
            data_url.to_string(),
            "data:text/html;charset=UTF-8,%E2%98%80%EF%B8%8F"
        );

        Ok(())
    }
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
    fn must_not_output_default_charset_if_bad_charset_given() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("text/html".to_string()));
        data_url.set_charset(Some("utf8".to_string())); // This must set charset to UTF-8
        data_url.set_charset(Some("BAD-CHARSET".to_string())); // And this bad input must make it fall back to US-ASCII
        data_url.set_is_base64_encoded(true);

        assert_eq!(data_url.to_string(), "data:text/html;base64,");

        Ok(())
    }
}
