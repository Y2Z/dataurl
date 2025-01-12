//
//  ██████╗  █████╗ ███████╗███████╗██╗███╗   ██╗ ██████╗
//  ██╔══██╗██╔══██╗██╔════╝██╔════╝██║████╗  ██║██╔════╝
//  ██████╔╝███████║███████╗███████╗██║██╔██╗ ██║██║  ███╗
//  ██╔═══╝ ██╔══██║╚════██║╚════██║██║██║╚██╗██║██║   ██║
//  ██║     ██║  ██║███████║███████║██║██║ ╚████║╚██████╔╝
//  ╚═╝     ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝
//

#[cfg(test)]
mod passing {
    use assert_cmd::prelude::*;
    use std::process::Command;

    #[test]
    fn must_properly_read_and_encode_basic_text_file() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-i").arg("tests/_data_/text-file.txt").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must be empty
            .stdout("data:text/plain,some%20content%0A\n");
    }

    #[test]
    fn must_properly_read_and_base64_encode_basic_text_file() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd
            .arg("-b")
            .arg("-i")
            .arg("tests/_data_/text-file.txt")
            .assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must be empty
            .stdout("data:text/plain;base64,c29tZSBjb250ZW50Cg==\n");
    }

    #[test]
    fn must_properly_read_and_encode_image_file() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd.arg("-i").arg("tests/_data_/pixel.png").assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must be empty
            .stdout("data:image/png,%89PNG%0D%0A%1A%0A%00%00%00%0DIHDR%00%00%00%01%00%00%00%01%08%06%00%00%00%1F%15%C4%89%00%00%00%0AIDATx%01c%00%01%00%00%05%00%016%D0%88%DD%00%00%00%00IEND%AEB%60%82\n");
    }

    #[test]
    fn must_properly_read_and_base64_encode_image_file() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        let assert = cmd
            .arg("-b")
            .arg("-i")
            .arg("tests/_data_/pixel.png")
            .assert();

        assert
            // Exit code must be 0
            .success()
            // STDERR must be empty
            .stderr("")
            // STDOUT must be empty
            .stdout("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAACklEQVR4AWMAAQAABQABNtCI3QAAAABJRU5ErkJggg==\n");
    }
}
