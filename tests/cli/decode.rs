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

    #[test]
    fn must_properly_parse_and_output_gbk_encoded_data_urls() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd
            .arg("-d")
            .arg("data:;charset=gbk;base64,PbnjtqvKocnu29rK0LGmsLI=")
            .assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be completely empty
            .stderr("")
            // STDOUT must contain nothing but a newline
            .stdout("=广东省深圳市宝安\n");
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
            .stderr("error: DataUrlParseError\n")
            // STDOUT must be empty
            .stdout("");
    }
}
