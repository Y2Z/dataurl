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
    fn spaces_around_url() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse(" data:, a b ")?;
        assert_eq!(data_url.data(), " a b".as_bytes());
        Ok(())
    }

    #[test]
    fn no_media_type() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:,Hello,%20World!")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(!data_url.encoded());
        assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello, World!");
        assert_eq!(data_url.fragment(), None);
        Ok(())
    }

    #[test]
    fn query_and_empty_fragment() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;,Hello?World#")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(!data_url.encoded());
        assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello?World");
        assert_eq!(data_url.fragment(), Some("".to_string()));
        Ok(())
    }

    #[test]
    fn empty_query_empty_fragment() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;,Hello?#")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(!data_url.encoded());
        assert_eq!(String::from_utf8_lossy(data_url.data()), "Hello?");
        assert_eq!(data_url.fragment(), Some("".to_string()));
        Ok(())
    }

    #[test]
    fn utf8_charset_no_data() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=utf8;base64,")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "UTF-8".to_string());
        assert!(data_url.encoded());
        assert_eq!(String::from_utf8_lossy(data_url.data()), "");
        assert_eq!(data_url.fragment(), None);
        Ok(())
    }

    #[test]
    fn utf8_charset_emoji() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=utf8;base64,4piA77iP")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "UTF-8".to_string());
        assert!(data_url.encoded());
        assert_eq!(data_url.data(), [226, 152, 128, 239, 184, 143]);
        assert_eq!(data_url.fragment(), None);
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
    fn empty_str_input() -> Result<(), DataUrlParseError> {
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
    fn missing_media_type() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:base64,SGVsbG8sIHdvcmxkIQo=")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(!data_url.encoded());
        assert_eq!(
            String::from_utf8_lossy(data_url.data()),
            "SGVsbG8sIHdvcmxkIQo="
        );
        assert_eq!(data_url.fragment(), None);
        Ok(())
    }

    #[test]
    fn bad_charset() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:;charset=BAD-CHARSET;base64,")?;
        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert!(data_url.encoded());
        assert_eq!(data_url.data(), []);
        assert_eq!(data_url.fragment(), None);
        Ok(())
    }

    // #[test]
    // fn bad_media_type() -> Result<(), DataUrlParseError> {
    //     let data_url: DataUrl = DataUrl::parse("data:bad;,")?;
    //     assert_eq!(data_url.media_type(), "text/plain".to_string());
    //     assert_eq!(data_url.charset(), "US-ASCII".to_string());
    //     assert!(data_url.encoded());
    //     assert_eq!(data_url.data(), []);
    //     assert_eq!(data_url.fragment(), None);
    //     Ok(())
    // }
}
