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
            Arg::with_name("FILE")
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
    let input: Vec<u8> = if app.is_present("INPUT") {
        app.value_of("INPUT").unwrap().as_bytes().to_vec()
    } else if app.is_present("FILE") {
        match fs::read(app.value_of("FILE").unwrap()) {
            Ok(f) => f,
            Err(_) => {
                eprintln!(
                    "Error: unable to read input file {}.",
                    app.value_of("FILE").unwrap()
                );
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Error: no input provided.");
        vec![]
    };

    if is_in_decode_mode {
        let input_as_string: &str = std::str::from_utf8(&input).unwrap();
        std::process::exit(match DataUrl::parse(input_as_string) {
            Ok(data_url) => {
                println!("{}", data_url.get_text());
                0
            }
            Err(err) => {
                eprintln!("Error: {:?}.", err);
                1
            }
        });
    } else {
        let mut data_url = DataUrl::new();
        data_url.set_data(&input);
        if app.is_present("base64") {
            data_url.set_is_base64_encoded(true);
        }
        if app.is_present("charset") {
            let charset: &str = app.value_of("charset").unwrap();
            let success: bool = data_url.set_charset(Some(charset.to_string()));

            if !success {
                eprintln!("Error: invalid charset {}.", charset);
                std::process::exit(1);
            }
        }
        if app.is_present("media_type") {
            let media_type: &str = app.value_of("media_type").unwrap();
            let success: bool = data_url.set_media_type(Some(media_type.to_string()));

            if !success {
                eprintln!("Error: invalid media type {}.", media_type);
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
