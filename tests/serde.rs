//! The serde feature (run with `cargo test --features serde`)
#![cfg(feature = "serde")]

use dataurl::{DataEncoding, DataUrl, MimeType};
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

#[test]
fn data_url_as_string() {
    let data_url = DataUrl::parse("data:text/plain;charset=utf-8,Hello,%20World!").unwrap();
    assert_tokens(
        &data_url,
        &[Token::Str("data:text/plain;charset=utf-8,Hello,%20World!")],
    );
}

#[test]
fn data_url_keeps_base64() {
    let data_url: DataUrl = "data:image/png;base64,iVBORw==".parse().unwrap();
    assert_eq!(data_url.data_encoding(), DataEncoding::Base64);
    assert_tokens(&data_url, &[Token::Str("data:image/png;base64,iVBORw==")]);
}

#[test]
fn data_url_from_borrowed_and_owned_strings() {
    let data_url = DataUrl::parse("data:,x").unwrap();
    assert_de_tokens(&data_url, &[Token::BorrowedStr("data:,x")]);
    assert_de_tokens(&data_url, &[Token::String("data:,x")]);
}

#[test]
fn data_url_gets_normalized() {
    // Deserializing parses the string, so what gets serialized again is the normalized form
    let data_url: DataUrl = "DATA:Text/Plain;Charset=\"UTF-8\",x".parse().unwrap();
    assert_de_tokens(
        &data_url,
        &[Token::Str("DATA:Text/Plain;Charset=\"UTF-8\",x")],
    );
    assert_tokens(&data_url, &[Token::Str("data:text/plain;charset=UTF-8,x")]);
}

#[test]
fn invalid_data_url() {
    assert_de_tokens_error::<DataUrl>(
        &[Token::Str("https://example.com/")],
        "expected a data URL: not a data URL",
    );
    assert_de_tokens_error::<DataUrl>(
        &[Token::Str("data:;base64,a")],
        "expected a data URL: data URL contains invalid base64",
    );
}

#[test]
fn data_url_must_be_a_string() {
    assert_de_tokens_error::<DataUrl>(
        &[Token::U32(1)],
        "invalid type: integer `1`, expected a data URL",
    );
}

#[test]
fn mime_type_as_string() {
    let mime_type = MimeType::parse("Text/HTML; charset=\"Shift_JIS\"").unwrap();
    assert_tokens(&mime_type, &[Token::Str("text/html;charset=Shift_JIS")]);
    assert_de_tokens(
        &mime_type,
        &[Token::Str("Text/HTML; charset=\"Shift_JIS\"")],
    );
}

#[test]
fn invalid_mime_type() {
    assert_de_tokens_error::<MimeType>(
        &[Token::Str("text")],
        "expected a MIME type: invalid media type",
    );
}
