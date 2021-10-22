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
    fn must_be_us_none_by_default() -> Result<(), DataUrlParseError> {
        let data_url = DataUrl::new();

        assert_eq!(data_url.get_charset_no_default(), None);

        Ok(())
    }

    #[test]
    fn must_be_possible_to_set_to_utf8() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        assert!(data_url.set_charset(Some("utf8".to_string())));

        assert_eq!(data_url.get_charset_no_default(), Some("UTF-8".to_string()));

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
    fn must_fall_back_to_none_if_given_bad_charset() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        // This bad input must make it fall back to US-ASCII
        assert!(!data_url.set_charset(Some("BAD-CHARSET".to_string())));

        assert_eq!(data_url.get_charset_no_default(), None);

        Ok(())
    }
}
