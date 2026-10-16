use dataurl::DataUrl;
use std::ffi::OsString;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::process::exit;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() -> String {
    format!("Usage: {NAME} [OPTIONS] [INPUT]")
}

fn help() -> String {
    format!(
        "{NAME} {VERSION}
{description}

{usage}

Arguments:
  [INPUT]  Text to encode, or data URL to decode (read from STDIN if omitted)

Options:
  -b, --base64                Encode data using base64
  -c, --charset <CHARSET>     Set charset (INPUT text gets converted to it)
  -d, --decode                Decode a data URL and output its data
  -f, --fragment <FRAGMENT>   Append URL fragment
  -i, --input-file <FILE>     Read input from FILE (\"-\" for STDIN)
  -o, --output-file <FILE>    Write output to FILE (\"-\" for STDOUT)
  -t, --media-type <TYPE>     Set media type
  -h, --help                  Print help
  -V, --version               Print version
",
        description = env!("CARGO_PKG_DESCRIPTION"),
        usage = usage(),
    )
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    exit(1);
}

fn fail_with_usage(message: &str) -> ! {
    eprintln!(
        "error: {message}\n\n{}\n\nFor more information, try '--help'.",
        usage()
    );
    exit(2);
}

#[derive(Default)]
struct Options {
    base64: bool,
    charset: Option<String>,
    decode: bool,
    fragment: Option<String>,
    input_file: Option<OsString>,
    output_file: Option<OsString>,
    media_type: Option<String>,
    input: Option<String>,
}

fn parse_args(args: impl Iterator<Item = OsString>) -> Options {
    let mut options = Options::default();
    let mut args = args.peekable();
    let mut only_positional = false;

    fn into_string(value: OsString) -> String {
        value
            .into_string()
            .unwrap_or_else(|_| fail("Argument is not valid UTF-8"))
    }

    while let Some(arg) = args.next() {
        let arg_str = arg.to_string_lossy().to_string();

        if only_positional || arg_str == "-" || !arg_str.starts_with('-') {
            if options.input.is_some() {
                fail_with_usage(&format!("unexpected argument '{arg_str}'"));
            }
            options.input = Some(into_string(arg));
            continue;
        }

        if arg_str == "--" {
            only_positional = true;
            continue;
        }

        // Support both "--option value" and "--option=value", as well as "-o value" and "-ovalue"
        let (flag, inline_value): (String, Option<String>) = if arg_str.starts_with("--") {
            match arg_str.split_once('=') {
                Some((flag, value)) => (flag.to_string(), Some(value.to_string())),
                None => (arg_str.clone(), None),
            }
        } else if arg_str.chars().count() > 2 {
            let mut chars = arg_str.chars();
            let flag: String = chars.by_ref().take(2).collect();
            (flag, Some(chars.collect()))
        } else {
            (arg_str.clone(), None)
        };

        let mut value = |name: &str| -> OsString {
            match &inline_value {
                Some(value) => OsString::from(value),
                None => args.next().unwrap_or_else(|| {
                    fail_with_usage(&format!("a value is required for '{name}'"))
                }),
            }
        };

        match flag.as_str() {
            "-b" | "--base64" | "-d" | "--decode" | "-h" | "--help" | "-V" | "--version"
                if inline_value.is_some() =>
            {
                fail_with_usage(&format!("unexpected value for '{flag}'"))
            }
            "-b" | "--base64" => options.base64 = true,
            "-d" | "--decode" => options.decode = true,
            "-h" | "--help" => {
                print!("{}", help());
                exit(0);
            }
            "-V" | "--version" => {
                println!("{NAME} {VERSION}");
                exit(0);
            }
            "-c" | "--charset" => options.charset = Some(into_string(value("--charset"))),
            "-f" | "--fragment" => options.fragment = Some(into_string(value("--fragment"))),
            "-i" | "--input-file" => options.input_file = Some(value("--input-file")),
            "-o" | "--output-file" => options.output_file = Some(value("--output-file")),
            "-t" | "--media-type" => options.media_type = Some(into_string(value("--media-type"))),
            _ => fail_with_usage(&format!("unexpected argument '{arg_str}'")),
        }
    }

    options
}

fn write_output(output_file: &Option<OsString>, data: &[u8]) {
    let result = match output_file {
        Some(path) => fs::write(path, data).map_err(|e| (e, path.to_string_lossy())),
        None => {
            let mut stdout = io::stdout().lock();
            match stdout.write_all(data).and_then(|_| stdout.flush()) {
                // Being piped into something like `head` is not an error
                Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
                result => result.map_err(|e| (e, "STDOUT".into())),
            }
        }
    };

    if let Err((e, destination)) = result {
        fail(&format!("Unable to write to '{destination}': {e}"));
    }
}

fn main() {
    let mut options = parse_args(std::env::args_os().skip(1));

    // "-" means STDIN/STDOUT, same as not specifying the option at all
    options.input_file = options.input_file.filter(|path| path != "-");
    options.output_file = options.output_file.filter(|path| path != "-");

    if options.input.is_some() && options.input_file.is_some() {
        fail("Both file and argument inputs provided");
    }

    let input: Vec<u8> = if let Some(input) = &options.input {
        input.as_bytes().to_vec()
    } else if let Some(path) = &options.input_file {
        fs::read(path).unwrap_or_else(|e| {
            fail(&format!(
                "Unable to read input file '{}': {e}",
                path.to_string_lossy()
            ))
        })
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        if let Err(e) = io::stdin().lock().read_to_end(&mut buffer) {
            fail(&format!("Unable to read STDIN: {e}"));
        }
        buffer
    };

    if options.decode {
        let data_url = DataUrl::parse(&String::from_utf8_lossy(&input))
            .unwrap_or_else(|e| fail(&e.to_string()));

        if options.output_file.is_none() && io::stdout().is_terminal() && !data_url.is_binary() {
            // Text printed into a terminal gets converted from the data URL's charset
            // TODO: check the terminal's encoding instead of assuming it's UTF-8
            write_output(&None, data_url.text().as_bytes());
        } else {
            write_output(&options.output_file, data_url.data());
        }
    } else {
        let mut data_url = DataUrl::new();

        data_url.set_is_base64_encoded(options.base64);

        if let Some(media_type) = &options.media_type {
            data_url
                .set_media_type(Some(media_type))
                .unwrap_or_else(|_| fail(&format!("Invalid media type '{media_type}'")));
        }

        match (&options.charset, &options.input) {
            (Some(charset), text) => {
                data_url
                    .set_charset(Some(charset))
                    .unwrap_or_else(|_| fail(&format!("Invalid encoding '{charset}'")));

                match text {
                    // Text given as an argument is UTF-8, so it gets converted to the charset
                    Some(text) => data_url.set_text(text).unwrap_or_else(|_| {
                        fail(&format!(
                            "Input text can't be represented in {}",
                            data_url.charset()
                        ))
                    }),
                    // Files and STDIN are taken as-is
                    None => data_url.set_data(&input),
                }
            }
            (None, Some(text)) => {
                data_url.set_data(text.as_bytes());
                if !text.is_ascii() {
                    data_url.set_charset(Some("UTF-8")).unwrap();
                }
            }
            (None, None) => data_url.set_data(&input),
        }

        if let Some(fragment) = &options.fragment {
            data_url.set_fragment(Some(fragment));
        }

        write_output(&options.output_file, format!("{data_url}\n").as_bytes());
    }
}
