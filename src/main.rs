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

    //////////////////////////////////////////////////////////////////////////

    let is_in_decode_mode: bool = app.is_present("decode");
    let has_arg_input: bool = app.is_present("INPUT");
    let mut has_file_input: bool = app.is_present("FILE");
    let input_file_path: &str = if has_file_input {
        app.value_of("FILE").unwrap()
    } else {
        ""
    };
    if has_file_input && input_file_path == "-" {
        has_file_input = false;
    }

    //////////////////////////////////////////////////////////////////////////

    if has_arg_input && has_file_input {
        eprintln!("error: Both file and argument inputs provided");
        std::process::exit(1);
    }

    //////////////////////////////////////////////////////////////////////////

    let input: Vec<u8> = if has_arg_input {
        app.value_of("INPUT").unwrap().as_bytes().to_vec()
    } else if has_file_input {
        match fs::read(input_file_path) {
            Ok(input_file_data) => input_file_data,
            Err(_) => {
                eprintln!("error: Unable to read input file '{}'", input_file_path);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("error: No input provided");
        vec![]
    };

    //////////////////////////////////////////////////////////////////////////

    if is_in_decode_mode {
        // TODO: ideally the program needs to check the current terminal locale (encoding), and not just assume it's UTF-8
        let input_as_string: String = String::from_utf8_lossy(&input).to_string();

        std::process::exit(match DataUrl::parse(&input_as_string) {
            Ok(data_url) => {
                println!("{}", data_url.get_text());
                0
            }
            Err(err) => {
                eprintln!("error: {:?}", err);
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
                eprintln!("error: Invalid charset '{}'", charset);
                std::process::exit(1);
            }
        } else {
            // TODO: ideally the program needs to check the current terminal locale (encoding), and not just assume it's UTF-8

            // Automatically enforce ;charset=UTF-8 for non-ascii argument inputs
            if has_arg_input && !String::from_utf8_lossy(&input).to_string().is_ascii() {
                data_url.set_charset(Some("UTF-8".to_string()));
            }
        }

        if app.is_present("media_type") {
            let media_type: &str = app.value_of("media_type").unwrap();
            let success: bool = data_url.set_media_type(Some(media_type.to_string()));

            if !success {
                eprintln!("error: Invalid media type '{}'", media_type);
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
