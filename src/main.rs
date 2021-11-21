extern crate clap;

use atty::Stream;
use clap::{crate_authors, crate_description, crate_version, App, Arg};
use dataurl::DataUrl;
use std::env;
use std::fs;
use std::io::{self, prelude::*, Write};

pub fn read_stdin() -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![];

    match io::stdin().lock().read_to_end(&mut buffer) {
        Ok(_) => buffer,
        Err(_) => buffer,
    }
}

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
            Arg::with_name("ENCODING")
                .short("c")
                .long("charset")
                .multiple(false)
                .takes_value(true)
                .help("Sets custom encoding parameter"),
        )
        .arg(
            Arg::with_name("decode")
                .short("d")
                .long("decode")
                .multiple(false)
                .help("Toggles decode mode on"),
        )
        .arg(
            Arg::with_name("FRAGMENT")
                .short("f")
                .long("fragment")
                .multiple(false)
                .takes_value(true)
                .help("Appends URL fragment"),
        )
        .arg(
            Arg::with_name("INPUT FILE")
                .short("i")
                .long("input-file")
                .multiple(false)
                .takes_value(true)
                .help("Provides input file"),
        )
        .arg(
            Arg::with_name("OUTPUT FILE")
                .short("o")
                .long("output-file")
                .multiple(false)
                .takes_value(true)
                .help("Specifies output file"),
        )
        .arg(
            Arg::with_name("MEDIA TYPE")
                .short("t")
                .long("media-type")
                .multiple(false)
                .takes_value(true)
                .help("Sets custom media type"),
        )
        .arg(Arg::with_name("INPUT").help("Input string").required(false))
        .get_matches();

    //////////////////////////////////////////////////////////////////////////

    let decode_mode_enabled: bool = app.is_present("decode");
    let string_input_set: bool = app.is_present("INPUT");
    // let stdin_is_a_tty: bool = !io::stdio::stdin_raw().isatty();
    let stdout_is_a_tty: bool = atty::is(Stream::Stdout);
    let mut file_input_set: bool = app.is_present("INPUT FILE");
    let mut file_output_set: bool = app.is_present("OUTPUT FILE");
    let input_file_path: &str = if file_input_set {
        app.value_of("INPUT FILE").unwrap()
    } else {
        "-"
    };
    let output_file_path: &str = if file_output_set {
        app.value_of("OUTPUT FILE").unwrap()
    } else {
        "-"
    };
    if file_input_set && input_file_path == "-" {
        file_input_set = false;
    }
    if file_output_set && output_file_path == "-" {
        file_output_set = false;
    }
    let file_input_set = file_input_set;
    let file_output_set = file_output_set;

    //////////////////////////////////////////////////////////////////////////

    if string_input_set && file_input_set {
        eprintln!("error: Both file and argument inputs provided");
        std::process::exit(1);
    }

    if !stdout_is_a_tty && file_output_set {
        eprintln!("error: Both stdout and argument output provided");
        std::process::exit(1);
    }

    //////////////////////////////////////////////////////////////////////////

    let input: Vec<u8> = if string_input_set {
        app.value_of("INPUT").unwrap().as_bytes().to_vec()
    } else if file_input_set {
        match fs::read(input_file_path) {
            Ok(input_file_data) => input_file_data,
            Err(_) => {
                eprintln!("error: Unable to read input file '{}'", input_file_path);
                std::process::exit(1);
            }
        }
    } else {
        // TODO: make it hang here, waiting on input from STDIN the way GNU's `base64` or `cat` do
        read_stdin()
    };

    //////////////////////////////////////////////////////////////////////////

    if decode_mode_enabled {
        // TODO: ideally the program needs to check the current terminal locale (encoding), and not just assume it's UTF-8
        let input_as_string: String = String::from_utf8_lossy(&input).to_string();

        std::process::exit(match DataUrl::parse(&input_as_string) {
            Ok(data_url) => {
                if !stdout_is_a_tty || file_output_set || data_url.is_binary() {
                    // Write raw bytes if the output is a file, or if the contents of this data URL has binary format
                    if file_output_set {
                        let mut handle = fs::File::create(output_file_path).unwrap();
                        handle.write_all(data_url.data()).unwrap();
                    } else {
                        let stdout = io::stdout();
                        let mut handle = stdout.lock();
                        handle.write_all(data_url.data()).unwrap();
                    }
                } else {
                    // When printing the result directly into the terminal, we have to convert data into UTF-8 (must account for non-US-ASCII/UTF-8 charsets)
                    print!("{}", data_url.text());
                }
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

        if app.is_present("ENCODING") {
            let charset: &str = app.value_of("ENCODING").unwrap();
            let success: bool = data_url.set_charset(Some(charset.to_string()));

            if !success {
                eprintln!("error: Invalid encoding '{}'", charset);
                std::process::exit(1);
            }

            // TODO: encode data into provided charset, if different
        } else {
            // TODO: ideally the program needs to check the current terminal locale (encoding), and not just assume it's UTF-8

            // Automatically enforce ;charset=UTF-8 for non-ascii argument inputs
            if string_input_set && !String::from_utf8_lossy(&input).to_string().is_ascii() {
                data_url.set_charset(Some("UTF-8".to_string()));
            }
        }

        if app.is_present("MEDIA TYPE") {
            let media_type: &str = app.value_of("MEDIA TYPE").unwrap();
            let success: bool = data_url.set_media_type(Some(media_type.to_string()));

            if !success {
                eprintln!("error: Invalid media type '{}'", media_type);
                std::process::exit(1);
            }
        }

        if app.is_present("FRAGMENT") {
            data_url.set_fragment(Some(app.value_of("FRAGMENT").unwrap().to_string()));
        }

        println!("{}", data_url.to_string());

        std::process::exit(0);
    }
}
