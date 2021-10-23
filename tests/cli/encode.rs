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

    #[test]
    fn must_support_setting_media_type() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-b").arg("-t").arg("text/html").arg(" ").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:text/html;base64,IA==\n");
    }

    #[test]
    fn must_support_setting_charset() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-b").arg("-c").arg("utf8").arg(" ").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:;charset=UTF-8;base64,IA==\n");
    }

    #[test]
    fn must_set_fragment_if_provided() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-b").arg("-f").arg("something").arg(" ").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:;base64,IA==#something\n");
    }

    #[test]
    fn must_set_empty_fragment_if_provided() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-b").arg("-f").arg("").arg(" ").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must contain generated data URL
            .stdout("data:;base64,IA==#\n");
    }

    #[test]
    fn must_support_gbk_encoded_data_urls() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-c").arg("gbk").arg("Ü").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be completely empty
            .stderr("")
            // STDOUT must contain properly encoded data URL
            .stdout("data:;charset=GBK,%26%23220%3B\n");
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
        let assert = cmd.arg("-t").arg("wrong/media/type").arg("Ü").assert();

        assert
            // Exit code must be 1
            .failure()
            // STDERR must contain error message
            .stderr("error: Invalid media type 'wrong/media/type'\n")
            // STDOUT must be empty
            .stdout("");
    }

    #[test]
    fn must_not_allow_incorrect_charset_to_be_set() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-c").arg("BAD-CHARSET").arg("Ü").assert();

        assert
            // Exit code must be 1
            .failure()
            // STDERR must contain error message
            .stderr("error: Invalid charset 'BAD-CHARSET'\n")
            // STDOUT must be empty
            .stdout("");
    }
}
