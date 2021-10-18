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
    fn default_and_set_fragment() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        // Should be none by default
        assert_eq!(data_url.fragment(), None);
        assert_eq!(data_url.to_string(), "data:,");

        // Shown as defined but empty is set to an empty string
        data_url.set_fragment(Some("".to_string()));
        assert_eq!(data_url.fragment(), Some("".to_string()));
        assert_eq!(data_url.to_string(), "data:,#");

        // Should be possible te unset it
        data_url.set_fragment(None);
        assert_eq!(data_url.fragment(), None);

        // Should be possible to set it to a string
        data_url.set_fragment(Some("something".to_string()));
        assert_eq!(data_url.fragment(), Some("something".to_string()));
        assert_eq!(data_url.to_string(), "data:,#something");

        // Should be possible to set it to a whitespace
        data_url.set_fragment(Some(" ".to_string()));
        assert_eq!(data_url.fragment(), Some(" ".to_string()));
        assert_eq!(data_url.to_string(), "data:,#%20");

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
