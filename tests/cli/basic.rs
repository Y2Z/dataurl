//  ██████╗  █████╗ ███████╗███████╗██╗███╗   ██╗ ██████╗
//  ██╔══██╗██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝
//  ██████╔╝███████║███████╗███████╗██║██╔██╗ ██║██║  ███╗
//  ██╔═══╝ ██╔══██║╚════██║╚════██║██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║███████║███████║██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝

#[cfg(test)]
mod passing {
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn must_print_help_information_out_when_asked_to() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-h").assert();

        let help_message: String = format!(
            "{bin} {ver}

Sunshine <sunshine@uberspace.net>
CLI tool and Rust crate for parsing and generating data URLs

USAGE:
    {bin}{exe} [FLAGS] [OPTIONS] [INPUT]

FLAGS:
    -b, --base64     Enforces base64 encoding
    -d, --decode     Toggles decode mode on
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --charset <ENCODING>           Sets custom encoding parameter
    -f, --fragment <FRAGMENT>          Appends URL fragment
    -i, --input-file <INPUT FILE>      Provides input file
    -t, --media-type <MEDIA TYPE>      Sets custom media type
    -o, --output-file <OUTPUT FILE>    Specifies output file

ARGS:
    <INPUT>    Input string
",
            bin = env!("CARGO_PKG_NAME"),
            ver = env!("CARGO_PKG_VERSION"),
            exe = if cfg!(windows) { ".exe" } else { "" }
        );

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain program name, version, and usage information
            .stdout(help_message);
    }

    #[test]
    fn must_print_program_name_and_version_number_when_asked_to() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-V").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain program name and version
            .stdout(format!(
                "{bin} {ver}\n",
                bin = env!("CARGO_PKG_NAME"),
                ver = env!("CARGO_PKG_VERSION")
            ));
    }
}

//  ███████╗ █████╗ ██╗██╗     ██╗███╗   ██╗ ██████╗
//  ██╔════╝██╔══██╗██║██║     ██║████╗  ██║██╔════╝
//  █████╗  ███████║██║██║     ██║██╔██╗ ██║██║  ███╗
//  ██╔══╝  ██╔══██║██║██║     ██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║██║███████╗██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝

#[cfg(test)]
mod failing {
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn must_fail_when_given_wrong_argument() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-X").arg("").assert();

        assert
            // Exit code must be 1
            .failure()
            // STDERR must contain error message
            .stderr(format!(
                "error: Found argument '-X' which wasn't expected, or isn't valid in this context

USAGE:
    {bin}{exe} [FLAGS] [OPTIONS] [INPUT]

For more information try --help\n",
                bin = env!("CARGO_PKG_NAME"),
                exe = if cfg!(windows) { ".exe" } else { "" }
            ))
            // STDOUT must contain absolutely nothing
            .stdout("");
    }

    #[test]
    fn must_fail_when_both_file_and_argument_input_given() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd
            .arg("-i")
            .arg("_data_/text-file.txt")
            .arg("text")
            .assert();

        assert
            // Exit code must be 1
            .failure()
            // STDERR must contain error message
            .stderr("error: Both file and argument inputs provided\n")
            // STDOUT must contain absolutely nothing
            .stdout("");
    }
}
