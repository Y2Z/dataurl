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
    fn default() -> Result<(), DataUrlParseError> {
        let data_url = DataUrl::new();
        assert_eq!(data_url.media_type_no_default(), None);
        Ok(())
    }

    #[test]
    fn utf8() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_media_type(Some("image/png".to_string()));
        assert_eq!(
            data_url.media_type_no_default(),
            Some("image/png".to_string())
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
    fn fallback_to_default_if_empty() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_media_type(Some("".to_string()));
        assert_eq!(data_url.media_type_no_default(), None);
        Ok(())
    }

    #[test]
    fn fallback_to_default_if_whitespace() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_media_type(Some(" ".to_string()));
        assert_eq!(data_url.media_type_no_default(), None);
        Ok(())
    }
}
