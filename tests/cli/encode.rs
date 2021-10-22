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
    fn must_generate_empty_data_url_when_empty_arg_input_is_given() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let assert = cmd.arg("").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:,\n");
    }

    #[test]
    fn must_generate_data_url_when_basic_arg_input_is_given() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let assert = cmd.arg("Hello, world!").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:,Hello%2C%20world%21\n");
    }

    #[test]
    fn must_generate_empty_base64_encoded_data_url_when_b_flag_and_empty_arg_input_are_given() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let assert = cmd.arg("-b").arg("").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:;base64,\n");
    }

    #[test]
    fn must_generate_short_base64_encoded_data_url_when_b_flag_and_whitespace_arg_input_are_given()
    {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let assert = cmd.arg("-b").arg(" ").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:;base64,IA==\n");
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
    fn must_not_allow_incorrect_media_type_to_be_set() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd
            .arg("-t")
            .arg("wrong/media/type")
            .arg("something")
            .assert();

        assert
            // Exit code must be 1
            .failure()
            // STDERR must contain error message
            .stderr("Error: invalid media type.\n")
            // STDOUT must be empty
            .stdout("");
    }
}
