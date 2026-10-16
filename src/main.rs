use dataurl::{detect_media_type, extension_for_media_type, DataEncoding, DataUrl, Escaping};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
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
  -s, --shortest              Use base64 or percent-encoding, whichever is shorter
  -m, --minimal               Only escape what has to be when percent-encoding
                              (for double-quoted HTML attributes and CSS strings)
  -c, --charset <CHARSET>     Set charset (INPUT text gets converted to it)
  -f, --fragment <FRAGMENT>   Append URL fragment
  -t, --media-type <TYPE>     Set media type (detected for files and STDIN if omitted)
  -i, --input-file <FILE>     Read input from FILE (\"-\" for STDIN); can be given
                              several times, for one data URL per line
  -o, --output-file <FILE>    Write output to FILE (\"-\" for STDOUT)
      --size                  Print the lengths of the percent-encoded and base64-encoded
                              data URLs instead (followed by the file name, if any)
  -d, --decode                Decode a data URL and output its data
  -O, --output-dir <DIR>      Decode into files in DIR, named after the input files,
                              with extensions matching their media types
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
    shortest: bool,
    minimal: bool,
    size: bool,
    charset: Option<String>,
    decode: bool,
    fragment: Option<String>,
    input_files: Vec<OsString>,
    output_file: Option<OsString>,
    output_dir: Option<OsString>,
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
            "-b" | "--base64" | "-s" | "--shortest" | "-m" | "--minimal" | "--size" | "-d"
            | "--decode" | "-h" | "--help" | "-V" | "--version"
                if inline_value.is_some() =>
            {
                fail_with_usage(&format!("unexpected value for '{flag}'"))
            }
            "-b" | "--base64" => options.base64 = true,
            "-s" | "--shortest" => options.shortest = true,
            "-m" | "--minimal" => options.minimal = true,
            "--size" => options.size = true,
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
            "-i" | "--input-file" => options.input_files.push(value("--input-file")),
            "-o" | "--output-file" => options.output_file = Some(value("--output-file")),
            "-O" | "--output-dir" => options.output_dir = Some(value("--output-dir")),
            "-t" | "--media-type" => options.media_type = Some(into_string(value("--media-type"))),
            _ => fail_with_usage(&format!("unexpected argument '{arg_str}'")),
        }
    }

    options
}

fn write_output(
    output_file: &Option<OsString>,
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) {
    let result = match output_file {
        Some(path) => fs::File::create(path)
            .and_then(|file| {
                let mut writer = io::BufWriter::new(file);
                write(&mut writer)?;
                writer.flush()
            })
            .map_err(|e| (e, path.to_string_lossy())),
        None => {
            let mut stdout = io::stdout().lock();
            match write(&mut stdout).and_then(|_| stdout.flush()) {
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

/// Where a piece of input comes from
enum Source {
    Argument(String),
    File(OsString),
    Stdin,
}

impl Source {
    fn read(&self) -> Result<Vec<u8>, String> {
        match self {
            Source::Argument(text) => Ok(text.as_bytes().to_vec()),
            Source::File(path) => fs::read(path).map_err(|e| {
                format!(
                    "Unable to read input file '{}': {e}",
                    path.to_string_lossy()
                )
            }),
            Source::Stdin => {
                let mut buffer: Vec<u8> = Vec::new();
                io::stdin()
                    .lock()
                    .read_to_end(&mut buffer)
                    .map_err(|e| format!("Unable to read STDIN: {e}"))?;
                Ok(buffer)
            }
        }
    }

    /// The file name (without directories), used for detecting media types
    fn file_name(&self) -> Option<&str> {
        match self {
            Source::File(path) => Path::new(path).file_name()?.to_str(),
            _ => None,
        }
    }

    /// The file name without its extension, used for naming decoded files
    fn file_stem(&self) -> String {
        match self {
            Source::File(path) => Path::new(path)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| "data".to_string()),
            _ => "data".to_string(),
        }
    }
}

fn sources(options: &mut Options) -> Vec<Source> {
    if let Some(text) = options.input.take() {
        if !options.input_files.is_empty() {
            fail("Both file and argument inputs provided");
        }
        return vec![Source::Argument(text)];
    }

    if options.input_files.is_empty() {
        return vec![Source::Stdin];
    }

    let sources: Vec<Source> = options
        .input_files
        .drain(..)
        // "-" means STDIN, same as not specifying the option at all
        .map(|path| match path == "-" {
            true => Source::Stdin,
            false => Source::File(path),
        })
        .collect();

    if sources
        .iter()
        .filter(|s| matches!(s, Source::Stdin))
        .count()
        > 1
    {
        fail_with_usage("STDIN can only be read once");
    }

    sources
}

fn encode(options: &Options, source: &Source, input: Vec<u8>) -> DataUrl {
    let mut data_url = DataUrl::new();

    if options.base64 {
        data_url.set_data_encoding(DataEncoding::Base64);
    } else if options.shortest {
        data_url.set_data_encoding(DataEncoding::Shortest);
    }

    if options.minimal {
        data_url.set_escaping(Escaping::Minimal);
    }

    if let Some(media_type) = &options.media_type {
        data_url
            .set_media_type(Some(media_type))
            .unwrap_or_else(|_| fail(&format!("Invalid media type '{media_type}'")));
    } else if !matches!(source, Source::Argument(_)) {
        // Files and STDIN get their media type detected (text given as an argument has none)
        if let Some(media_type) = detect_media_type(&input, source.file_name()) {
            data_url.set_media_type(Some(media_type)).unwrap();
        }
    }

    match (&options.charset, source) {
        (Some(charset), source) => {
            data_url
                .set_charset(Some(charset))
                .unwrap_or_else(|_| fail(&format!("Invalid encoding '{charset}'")));

            match source {
                // Text given as an argument is UTF-8, so it gets converted to the charset
                Source::Argument(text) => data_url.set_text(text).unwrap_or_else(|_| {
                    fail(&format!(
                        "Input text can't be represented in {}",
                        data_url.charset()
                    ))
                }),
                // Files and STDIN are taken as-is
                _ => data_url.set_data(input),
            }
        }
        (None, Source::Argument(text)) => {
            if !text.is_ascii() {
                data_url.set_charset(Some("UTF-8")).unwrap();
            }
            data_url.set_data(input);
        }
        (None, _) => data_url.set_data(input),
    }

    if let Some(fragment) = &options.fragment {
        data_url.set_fragment(Some(fragment));
    }

    data_url
}

/// The extension for a decoded file: from the media type if it's a known one,
/// otherwise from the data's signature, otherwise "txt" or "bin"
fn extension_for(data_url: &DataUrl) -> &'static str {
    extension_for_media_type(data_url.media_type())
        .or_else(|| detect_media_type(data_url.data(), None).and_then(extension_for_media_type))
        .unwrap_or(if data_url.is_binary() { "bin" } else { "txt" })
}

/// Picks a file name that hasn't been used yet in this run: "name.ext", "name-2.ext", …
fn unique_file_name(stem: &str, extension: &str, used: &mut HashSet<String>) -> String {
    // Files like "logo.png.txt" become "logo.png" rather than "logo.png.png"
    let suffix = format!(".{extension}");
    let stem = match stem.len() > suffix.len()
        && stem.to_ascii_lowercase().ends_with(&suffix)
        && stem.is_char_boundary(stem.len() - suffix.len())
    {
        true => &stem[..stem.len() - suffix.len()],
        false => stem,
    };

    let mut name = format!("{stem}.{extension}");
    let mut number = 2;
    while !used.insert(name.clone()) {
        name = format!("{stem}-{number}.{extension}");
        number += 1;
    }
    name
}

fn decode(source: &Source) -> DataUrl {
    let input = source.read().unwrap_or_else(|message| fail(&message));
    DataUrl::parse(&String::from_utf8_lossy(&input)).unwrap_or_else(|e| match source {
        Source::File(path) => fail(&format!("{}: {e}", path.to_string_lossy())),
        _ => fail(&e.to_string()),
    })
}

fn main() {
    let mut options = parse_args(std::env::args_os().skip(1));

    // "-" means STDOUT, same as not specifying the option at all
    options.output_file = options.output_file.filter(|path| path != "-");

    if options.base64 && options.shortest {
        fail_with_usage("'--base64' and '--shortest' can't be used together");
    }
    if options.decode && options.size {
        fail_with_usage("'--size' can't be used together with '--decode'");
    }
    if options.output_dir.is_some() && !options.decode {
        fail_with_usage("'--output-dir' only works together with '--decode'");
    }
    if options.output_dir.is_some() && options.output_file.is_some() {
        fail_with_usage("'--output-file' and '--output-dir' can't be used together");
    }

    let sources = sources(&mut options);

    if let Some(output_dir) = &options.output_dir {
        let output_dir = Path::new(output_dir);
        fs::create_dir_all(output_dir).unwrap_or_else(|e| {
            fail(&format!(
                "Unable to create directory '{}': {e}",
                output_dir.display()
            ))
        });

        let mut used_names = HashSet::new();
        for source in &sources {
            let data_url = decode(source);
            let name = unique_file_name(
                &source.file_stem(),
                extension_for(&data_url),
                &mut used_names,
            );
            let path = output_dir.join(name);
            fs::write(&path, data_url.data())
                .unwrap_or_else(|e| fail(&format!("Unable to write to '{}': {e}", path.display())));
            // Report what got written where, which matters since the names are picked automatically
            write_output(&None, |writer| writeln!(writer, "{}", path.display()));
        }
    } else if options.decode {
        if sources.len() > 1 {
            fail_with_usage("decoding more than one input needs '--output-dir'");
        }
        let data_url = decode(&sources[0]);

        if options.output_file.is_none() && io::stdout().is_terminal() && !data_url.is_binary() {
            // Text printed into a terminal gets converted from the data URL's charset
            // TODO: check the terminal's encoding instead of assuming it's UTF-8
            write_output(&None, |writer| writer.write_all(data_url.text().as_bytes()));
        } else {
            write_output(&options.output_file, |writer| {
                writer.write_all(data_url.data())
            });
        }
    } else {
        write_output(&options.output_file, |writer| {
            for source in &sources {
                let input = source.read().unwrap_or_else(|message| {
                    // Keep what's been written so far
                    let _ = writer.flush();
                    fail(&message)
                });
                let mut data_url = encode(&options, source, input);

                if options.size {
                    data_url.set_data_encoding(DataEncoding::Percent);
                    let percent_encoded_length = data_url.encoded_len();
                    data_url.set_data_encoding(DataEncoding::Base64);
                    let base64_encoded_length = data_url.encoded_len();
                    write!(writer, "{percent_encoded_length}\t{base64_encoded_length}")?;
                    if let Source::File(path) = source {
                        write!(writer, "\t{}", path.to_string_lossy())?;
                    }
                    writeln!(writer)?;
                } else {
                    data_url.write_to(&mut *writer)?;
                    writer.write_all(b"\n")?;
                }
            }
            Ok(())
        });
    }
}
