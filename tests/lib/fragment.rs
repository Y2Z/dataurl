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
    fn must_be_none_by_default() -> Result<(), DataUrlParseError> {
        let data_url = DataUrl::new();

        assert_eq!(data_url.get_fragment(), None);

        Ok(())
    }

    #[test]
    fn must_be_rendered_as_pound_sign_if_set_to_an_empty_string() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_fragment(Some("".to_string()));

        assert_eq!(data_url.get_fragment(), Some("".to_string()));

        Ok(())
    }

    #[test]
    fn must_parse_empty_string_if_just_pound_sign_given() -> Result<(), DataUrlParseError> {
        let data_url = DataUrl::parse("data:,#")?;

        assert_eq!(data_url.get_fragment(), Some("".to_string()));

        Ok(())
    }

    #[test]
    fn must_be_possible_to_unset() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::parse("data:,#something")?;

        assert_eq!(data_url.get_fragment(), Some("something".to_string()));

        data_url.set_fragment(None);

        assert_eq!(data_url.get_fragment(), None);

        Ok(())
    }

    #[test]

    fn must_be_possible_to_set() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_fragment(Some("something".to_string()));

        assert_eq!(data_url.get_fragment(), Some("something".to_string()));

        Ok(())
    }

    #[test]
    fn must_be_possible_to_set_it_to_a_whitespace() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_fragment(Some(" ".to_string()));

        assert_eq!(data_url.get_fragment(), Some(" ".to_string()));

        Ok(())
    }

    #[test]
    fn must_remain_abset_if_given_none() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_fragment(None);

        assert_eq!(data_url.get_fragment(), None);

        Ok(())
    }
}
