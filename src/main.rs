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
                .help("Enforces base64 encoding"),
        )
        .arg(
            Arg::with_name("charset")
                .short("c")
                .long("charset")
                .multiple(false)
                .takes_value(true)
                .help("Sets custom charset"),
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
                .help("Appends URL fragment"),
        )
        .arg(
            Arg::with_name("input-file")
                .short("i")
                .long("input-file")
                .multiple(false)
                .takes_value(true)
                .help("Provides input file"),
        )
        .arg(
            Arg::with_name("media_type")
                .short("t")
                .long("media-type")
                .multiple(false)
                .takes_value(true)
                .help("Sets custom media type"),
        )
        .arg(Arg::with_name("INPUT").help("Input string").required(false))
        .get_matches();

    let is_in_decode_mode: bool = app.is_present("decode");
    let input = if app.is_present("INPUT") {
        app.value_of("INPUT").unwrap().to_string()
    } else if app.is_present("input-file") {
        fs::read_to_string(app.value_of("input-file").unwrap())
            .expect("Something went wrong while trying to read the file")
    } else {
        eprintln!("Error: no input provided.");
        "".to_string()
    };

    if is_in_decode_mode {
        std::process::exit(match DataUrl::parse(&input) {
            Ok(data_url) => {
                println!("{}", String::from_utf8_lossy(data_url.get_data()));
                0
            }
            Err(err) => {
                eprintln!("Error: {:?}.", err);
                1
            }
        });
    } else {
        let mut data_url = DataUrl::new();
        data_url.set_data(input.as_bytes());
        if app.is_present("base64") {
            data_url.set_is_base64_encoded(true);
        }
        if app.is_present("charset") {
            data_url.set_charset(Some(app.value_of("charset").unwrap().to_string()));
        }
        if app.is_present("media_type") {
            let success: bool =
                data_url.set_media_type(Some(app.value_of("media_type").unwrap().to_string()));
            if !success {
                eprintln!("Error: invalid media type.");
                std::process::exit(1);
            }
        }
        if app.is_present("fragment") {
            data_url.set_fragment(Some(app.value_of("fragment").unwrap().to_string()));
        }
        println!("{}", data_url.to_string());
        std::process::exit(0);
    }
}
