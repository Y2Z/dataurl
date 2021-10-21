extern crate clap;

use clap::{crate_authors, crate_description, crate_version, App, Arg};
use dataurl::DataUrl;
use std::env;
use std::fs;

fn main() {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(format!("\n{}", crate_authors!("\n")).as_str())
        .about(crate_description!())
        .arg(
            Arg::with_name("base64")
                .short("b")
                .long("base64")
                .multiple(false)
                .help("Enforce base64 encoding"),
        )
        .arg(
            Arg::with_name("charset")
                .short("c")
                .long("charset")
                .multiple(false)
                .takes_value(true)
                .help("Enforce custom charset"),
        )
        .arg(
            Arg::with_name("decode")
                .short("d")
                .long("decode")
                .multiple(false)
                .help("Toggles decode mode on"),
        )
        .arg(
            Arg::with_name("fragment")
                .short("f")
                .long("fragment")
                .multiple(false)
                .takes_value(true)
                .help("Append URL fragment"),
        )
        .arg(
            Arg::with_name("input-file")
                .short("i")
                .long("input-file")
                .multiple(false)
                .takes_value(true)
                .help("Append URL fragment"),
        )
        .arg(
            Arg::with_name("media_type")
                .short("t")
                .long("media-type")
                .multiple(false)
                .takes_value(true)
                .help("Sets custom media type for encode mode"),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Sets input string")
                .required(false)
                .index(1),
        )
        .get_matches();

    let is_in_decode_mode: bool = app.is_present("decode");
    let input = if app.is_present("INPUT") {
        app.value_of("INPUT").unwrap().to_string()
    } else if app.is_present("input-file") {
        fs::read_to_string(app.value_of("input-file").unwrap())
            .expect("Something went wrong reading the file")
    } else {
        eprintln!("no input provided");
        "".to_string()
    };

    if is_in_decode_mode {
        match DataUrl::parse(&input) {
            Ok(data_url) => {
                println!("{}", String::from_utf8_lossy(data_url.data()));
            }
            Err(_data_url_parse_err) => {
                eprintln!("parsing error");
            }
        }
    } else {
        let mut data_url = DataUrl::new();
        data_url.set_data(input.as_bytes());
        if app.is_present("base64") {
            data_url.set_base64_encoded(true);
        }
        if app.is_present("charset") {
            data_url.set_charset(Some(app.value_of("charset").unwrap().to_string()));
        }
        if app.is_present("media_type") {
            data_url.set_media_type(Some(app.value_of("media_type").unwrap().to_string()));
        }
        if app.is_present("fragment") {
            data_url.set_fragment(Some(app.value_of("fragment").unwrap().to_string()));
        }
        println!("{}", data_url.to_string());
    }
}
