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
    fn must_have_correct_code_for_the_readme_usage_snippet() -> Result<(), DataUrlParseError> {
        let data_url: DataUrl = DataUrl::parse("data:,Hello,%20World!")?;

        assert_eq!(data_url.media_type(), "text/plain".to_string());
        assert_eq!(data_url.media_type_no_default(), None);
        assert_eq!(data_url.charset(), "US-ASCII".to_string());
        assert_eq!(data_url.charset_no_default(), None);
        assert!(!data_url.is_base64_encoded());
        assert_eq!(
            data_url.data(),
            [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]
        );
        assert_eq!(data_url.fragment(), None);
        assert_eq!(data_url.to_string(), "data:,Hello%2C%20World%21");
        assert_eq!(data_url.text(), "Hello, World!");

        Ok(())
    }
}
