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
        assert_eq!(data_url.charset(), "US-ASCII");
        Ok(())
    }

    #[test]
    fn utf8() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_charset(Some("utf8".to_string()));
        assert_eq!(data_url.charset(), "UTF-8");
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
    fn fallback_to_default_if_bad() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();
        data_url.set_charset(Some("BAD-CHARSET".to_string())); // This bad input must make it fall back to US-ASCII
        assert_eq!(data_url.charset(), "US-ASCII");
        Ok(())
    }
}
