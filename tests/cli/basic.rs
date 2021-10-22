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

    const HELP_MESSAGE: &'static str = "dataurl 0.0.1

Sunshine <sunshine@uberspace.net>
CLI tool and Rust crate for parsing and generating data URLs

USAGE:
    dataurl [FLAGS] [OPTIONS] [INPUT]

FLAGS:
    -b, --base64     Enforce base64 encoding
    -d, --decode     Toggles decode mode on
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --charset <charset>          Enforce custom charset
    -f, --fragment <fragment>        Append URL fragment
    -i, --input-file <input-file>    Append URL fragment
    -t, --media-type <media_type>    Sets custom media type for encode mode

ARGS:
    <INPUT>    Sets input string
";

    #[test]
    fn must_print_help_information_out_when_asked_to() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-h").assert();

        assert
            // Exit code should be 0
            .success()
            // STDERR should be empty
            .stderr("")
            // STDOUT should contain program name, version, and usage information
            .stdout(HELP_MESSAGE);
    }

    #[test]
    fn must_print_program_name_and_version_number_when_asked_to() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-V").assert();

        assert
            // Exit code should be 0
            .success()
            // STDERR should be empty
            .stderr("")
            // STDOUT should contain program name and version
            .stdout(format!(
                "{} {}\n",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
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
            // Exit code should be 1
            .failure()
            // STDERR should contain error message
            .stderr(
                "error: Found argument '-X' which wasn't expected, or isn't valid in this context

USAGE:
    dataurl [FLAGS] [OPTIONS] [INPUT]

For more information try --help
",
            )
            // STDOUT should contain absolutely nothing
            .stdout("");
    }
}
