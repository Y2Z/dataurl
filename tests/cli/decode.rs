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
    fn must_parse_empty_data_url_arg_input_and_output_nothing_into_stdout_but_a_newline() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-d").arg("data:,").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be completely empty
            .stderr("")
            // STDOUT must contain nothing but a newline
            .stdout("\n");
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
    fn must_fail_if_given_empty_arg_input_and_output_error_message_into_stderr_and_nothing_into_stdout(
    ) {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-d").arg("").assert();

        assert
            // Exit code must be 1
            .failure()
            // STDERR must contain error message
            .stderr("Error: DataUrlParseError.\n")
            // STDOUT must be empty
            .stdout("");
    }
}
