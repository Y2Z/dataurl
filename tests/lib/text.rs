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

        assert_eq!(data_url.text(), "");

        Ok(())
    }

    #[test]
    fn must_remain_empty_after_given_empty_data() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_data(&[]);
        assert_eq!(data_url.text(), "");

        Ok(())
    }

    #[test]
    fn must_accept_and_return_same_ascii_text() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_data(b"some text");
        assert_eq!(data_url.text(), "some text");

        Ok(())
    }

    #[test]
    fn must_accept_and_return_same_utf8_text() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_charset(Some("utf8".to_string()));
        data_url.set_data("Ü".as_bytes());
        assert_eq!(data_url.text(), "Ü");

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
    fn must_return_garbage_when_given_unicode_data_without_setting_charset_to_utf8(
    ) -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_data("Ü".as_bytes());
        assert_eq!(data_url.text(), "Ãœ");

        Ok(())
    }
}
