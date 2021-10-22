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

        assert_eq!(data_url.get_media_type_no_default(), None);

        Ok(())
    }

    #[test]
    fn must_be_possible_to_set_to_image_png() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("image/png".to_string()));

        assert_eq!(
            data_url.get_media_type_no_default(),
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
    fn must_fall_back_to_none_if_set_to_empty() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some("".to_string()));

        assert_eq!(data_url.get_media_type_no_default(), None);

        Ok(())
    }

    #[test]
    fn must_fall_back_to_none_if_attempted_to_set_to_whitespace() -> Result<(), DataUrlParseError> {
        let mut data_url = DataUrl::new();

        data_url.set_media_type(Some(" ".to_string()));

        assert_eq!(data_url.get_media_type_no_default(), None);

        Ok(())
    }

    // #[test]
    // fn must_fall_back_to_none_if_attempted_to_set_to_bad() -> Result<(), DataUrlParseError> {
    //     let mut data_url = DataUrl::new();

    //     data_url.set_media_type(Some("bad".to_string()));
    //     assert_eq!(data_url.get_media_type_no_default(), None);

    //     Ok(())
    // }
}
