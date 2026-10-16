//! Web Platform Tests suites for data URLs (https://github.com/web-platform-tests/wpt).
//! See tests/wpt/README.md for where the fixtures come from.

mod common;

use common::json;
use dataurl::DataUrl;

#[test]
fn wpt_data_urls() {
    let fixture = json::parse(include_str!("wpt/data-urls.json"));
    assert!(!fixture.as_array().is_empty());

    for test in fixture.as_array() {
        let test = test.as_array();
        let input = test[0].as_str();

        match test[1].as_opt_str() {
            Some(expected_mime_type) => {
                let data_url = DataUrl::parse(input)
                    .unwrap_or_else(|e| panic!("{input:?} must be parsed, got {e:?}"));
                assert_eq!(data_url.mime_type(), expected_mime_type, "{input:?}");
                assert_eq!(data_url.data(), test[2].as_bytes(), "{input:?}");

                // Serializing and parsing again must yield the same result
                let reparsed = DataUrl::parse(&data_url.to_string()).unwrap();
                assert_eq!(reparsed.mime_type(), expected_mime_type, "{input:?}");
                assert_eq!(reparsed.data(), data_url.data(), "{input:?}");
            }
            None => assert!(DataUrl::parse(input).is_err(), "{input:?} must fail"),
        }
    }
}

#[test]
fn wpt_forgiving_base64() {
    let fixture = json::parse(include_str!("wpt/base64.json"));
    assert!(!fixture.as_array().is_empty());

    for test in fixture.as_array() {
        let test = test.as_array();
        let input = format!("data:;base64,{}", test[0].as_str());

        if test[1].is_null() {
            assert!(DataUrl::parse(&input).is_err(), "{input:?} must fail");
        } else {
            let data_url = DataUrl::parse(&input)
                .unwrap_or_else(|e| panic!("{input:?} must be parsed, got {e:?}"));
            assert_eq!(data_url.data(), test[1].as_bytes(), "{input:?}");
            assert_eq!(
                DataUrl::parse(&data_url.to_string()).unwrap().data(),
                data_url.data()
            );
        }
    }
}
