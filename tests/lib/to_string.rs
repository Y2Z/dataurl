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
    fn empty() -> Result<(), DataUrlParseError> {
        let data_url = DataUrl::new();
        assert_eq!(data_url.to_string(), "data:,");
        Ok(())
    }

    #[test]
    fn empty_encoded() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_encoded(true);
        assert_eq!(data_url.to_string(), "data:;base64,");
        Ok(())
    }

    #[test]
    fn empty_encoded_with_media_type() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_encoded(true);
        data_url.set_media_type(Some("text/plain".to_string()));
        assert_eq!(data_url.to_string(), "data:text/plain;base64,");
        Ok(())
    }

    #[test]
    fn empty_encoded_with_media_type_and_ascii_charset() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_encoded(true);
        data_url.set_media_type(Some("text/plain".to_string()));
        data_url.set_charset(Some("US-ASCII".to_string()));
        assert_eq!(data_url.to_string(), "data:text/plain;base64,");
        Ok(())
    }

    #[test]
    fn empty_encoded_with_media_type_and_utf8_charset() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_encoded(true);
        data_url.set_media_type(Some("text/plain".to_string()));
        data_url.set_charset(Some("utf8".to_string()));
        assert_eq!(
            data_url.to_string(),
            "data:text/plain;charset=UTF-8;base64,"
        );
        Ok(())
    }

    #[test]
    fn full_encoded_with_media_type_and_utf8_charset() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_encoded(true);
        data_url.set_media_type(Some("text/plain".to_string()));
        data_url.set_charset(Some("utf8".to_string()));
        data_url.set_data(&[0xe2, 0x98, 0x80, 0xef, 0xb8, 0x8f]); // Sun emoji in UTF-8 bytes
        assert_eq!(
            data_url.to_string(),
            "data:text/plain;charset=UTF-8;base64,4piA77iP"
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
    fn empty_encoded_with_media_type_and_bad_charset() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_encoded(true);
        data_url.set_media_type(Some("text/plain".to_string()));
        data_url.set_charset(Some("BAD-CHARSET".to_string())); // This bad input must make it fall back to US-ASCII
        assert_eq!(data_url.to_string(), "data:text/plain;base64,");
        Ok(())
    }
}
