use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn data_file(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "_data_", name]
        .iter()
        .collect()
}

fn run(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_dataurl"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin).unwrap();
    child.wait_with_output().unwrap()
}

#[track_caller]
fn assert_success(output: &Output, expected_stdout: &[u8]) {
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    assert_eq!(output.stdout, expected_stdout);
    assert!(output.status.success());
}

#[track_caller]
fn assert_failure(output: &Output, expected_stderr: &str) {
    assert_eq!(String::from_utf8_lossy(&output.stderr), expected_stderr);
    assert_eq!(output.stdout, b"");
    assert!(!output.status.success());
}

mod basic {
    use super::*;

    #[test]
    fn help() {
        let output = run(&["-h"], b"");
        let stdout = String::from_utf8(output.stdout.clone()).unwrap();
        assert!(stdout.starts_with(concat!("dataurl ", env!("CARGO_PKG_VERSION"), "\n")));
        assert!(stdout.contains("Usage: dataurl [OPTIONS] [INPUT]"));
        assert!(stdout.contains("--media-type <TYPE>"));
        assert_eq!(run(&["--help"], b"").stdout, output.stdout);
        assert!(output.status.success());
    }

    #[test]
    fn version() {
        assert_success(
            &run(&["-V"], b""),
            concat!("dataurl ", env!("CARGO_PKG_VERSION"), "\n").as_bytes(),
        );
    }

    #[test]
    fn unknown_argument() {
        let output = run(&["-X", ""], b"");
        assert_failure(
            &output,
            "error: unexpected argument '-X'\n\nUsage: dataurl [OPTIONS] [INPUT]\n\nFor more information, try '--help'.\n",
        );
        assert_eq!(output.status.code(), Some(2));
    }

    #[test]
    fn missing_option_value() {
        assert!(String::from_utf8_lossy(&run(&["-t"], b"").stderr)
            .starts_with("error: a value is required for '--media-type'"));
    }

    #[test]
    fn two_positional_inputs() {
        assert!(!run(&["a", "b"], b"").status.success());
    }

    #[test]
    fn file_and_argument_inputs() {
        let file = data_file("text-file.txt");
        assert_failure(
            &run(&["-i", file.to_str().unwrap(), "text"], b""),
            "error: Both file and argument inputs provided\n",
        );
    }

    #[test]
    fn unreadable_input_file() {
        let output = run(&["-i", "does/not/exist"], b"");
        assert!(String::from_utf8_lossy(&output.stderr)
            .starts_with("error: Unable to read input file 'does/not/exist'"));
        assert!(!output.status.success());
    }
}

mod encode {
    use super::*;

    #[test]
    fn empty_argument() {
        assert_success(&run(&[""], b""), b"data:,\n");
    }

    #[test]
    fn basic_argument() {
        assert_success(&run(&["Hello, world!"], b""), b"data:,Hello,%20world!\n");
    }

    #[test]
    fn non_ascii_argument_gets_utf8_charset() {
        assert_success(&run(&["Ü"], b""), b"data:;charset=UTF-8,%C3%9C\n");
    }

    #[test]
    fn base64() {
        assert_success(&run(&["-b", ""], b""), b"data:;base64,\n");
        assert_success(&run(&["-b", " "], b""), b"data:;base64,IA==\n");
        assert_success(
            &run(&["--base64", "\u{fb}\u{ff}"], b""),
            b"data:;charset=UTF-8;base64,w7vDvw==\n",
        );
    }

    #[test]
    fn media_type() {
        assert_success(
            &run(&["-b", "-t", "text/html", " "], b""),
            b"data:text/html;base64,IA==\n",
        );
        assert_success(
            &run(&["--media-type=TEXT/HTML", " "], b""),
            b"data:text/html,%20\n",
        );
        assert_success(&run(&["-ttext/css", "a{}"], b""), b"data:text/css,a{}\n");
    }

    #[test]
    fn invalid_media_type() {
        assert_failure(
            &run(&["-t", "wrong/media/type", "Ü"], b""),
            "error: Invalid media type 'wrong/media/type'\n",
        );
    }

    #[test]
    fn charset() {
        assert_success(
            &run(&["-b", "-c", "utf8", " "], b""),
            b"data:;charset=UTF-8;base64,IA==\n",
        );
    }

    #[test]
    fn charset_converts_argument_text() {
        assert_success(
            &run(&["-c", "gbk", "你好"], b""),
            b"data:;charset=GBK,%C4%E3%BA%C3\n",
        );
    }

    #[test]
    fn charset_unable_to_represent_argument_text() {
        assert_failure(
            &run(&["-c", "gbk", "Ü"], b""),
            "error: Input text can't be represented in GBK\n",
        );
    }

    #[test]
    fn charset_keeps_stdin_bytes_as_is() {
        assert_success(
            &run(&["-c", "gbk"], b"\xC4\xE3"),
            b"data:;charset=GBK,%C4%E3\n",
        );
    }

    #[test]
    fn invalid_charset() {
        assert_failure(
            &run(&["-c", "BAD-CHARSET", "Ü"], b""),
            "error: Invalid encoding 'BAD-CHARSET'\n",
        );
    }

    #[test]
    fn fragment() {
        assert_success(
            &run(&["-b", "-f", "something", " "], b""),
            b"data:;base64,IA==#something\n",
        );
        assert_success(&run(&["-b", "-f", "", " "], b""), b"data:;base64,IA==#\n");
    }

    #[test]
    fn stdin() {
        assert_success(&run(&[], b"from stdin"), b"data:,from%20stdin\n");
        assert_success(&run(&["-i", "-"], b"from stdin"), b"data:,from%20stdin\n");
    }

    #[test]
    fn input_file() {
        let file = data_file("text-file.txt");
        let contents = std::fs::read(&file).unwrap();
        let output = run(&["-i", file.to_str().unwrap()], b"");
        assert!(output.status.success());
        let data_url = dataurl::DataUrl::parse(&String::from_utf8(output.stdout).unwrap()).unwrap();
        assert_eq!(data_url.data(), contents);
        assert_eq!(data_url.media_type_no_default(), Some("text/plain"));
    }

    #[test]
    fn media_type_detected_for_input_file() {
        let file = data_file("lizard.png");
        let output = run(&["-b", "-i", file.to_str().unwrap()], b"");
        assert!(output.status.success());
        assert!(output.stdout.starts_with(b"data:image/png;base64,iVBOR"));
    }

    #[test]
    fn media_type_detected_for_stdin() {
        let png = std::fs::read(data_file("lizard.png")).unwrap();
        let output = run(&["-b"], &png);
        assert!(output.status.success());
        assert!(output.stdout.starts_with(b"data:image/png;base64,iVBOR"));
    }

    #[test]
    fn media_type_option_overrides_detection() {
        let file = data_file("lizard.png");
        let output = run(
            &[
                "-b",
                "-t",
                "application/octet-stream",
                "-i",
                file.to_str().unwrap(),
            ],
            b"",
        );
        assert!(output
            .stdout
            .starts_with(b"data:application/octet-stream;base64,iVBOR"));
    }

    #[test]
    fn media_type_not_detected_for_argument() {
        assert_success(
            &run(&["<html></html>"], b""),
            b"data:,%3Chtml%3E%3C/html%3E\n",
        );
    }

    #[test]
    fn output_file() {
        let path = std::env::temp_dir().join(format!("dataurl-test-{}.txt", std::process::id()));
        assert_success(&run(&["-o", path.to_str().unwrap(), "x"], b""), b"");
        assert_eq!(std::fs::read(&path).unwrap(), b"data:,x\n");
        std::fs::remove_file(path).unwrap();
    }
}

mod decode {
    use super::*;

    #[test]
    fn empty_data_url() {
        assert_success(&run(&["-d", "data:,"], b""), b"");
    }

    #[test]
    fn base64_png_round_trip() {
        let file = data_file("lizard.png");
        let png = std::fs::read(&file).unwrap();

        let encoded = run(
            &["-b", "-t", "image/png", "-i", file.to_str().unwrap()],
            b"",
        );
        assert!(encoded.status.success());
        assert_success(&run(&["-d"], &encoded.stdout), &png);
    }

    #[test]
    fn gbk_data_url_outputs_raw_bytes_when_piped() {
        let output = run(
            &["-d", "data:;charset=gbk;base64,PbnjtqvKocnu29rK0LGmsLI="],
            b"",
        );
        assert_success(
            &output,
            &[
                0x3D, 0xB9, 0xE3, 0xB6, 0xAB, 0xCA, 0xA1, 0xC9, 0xEE, 0xDB, 0xDA, 0xCA, 0xD0, 0xB1,
                0xA6, 0xB0, 0xB2,
            ],
        );
    }

    #[test]
    fn stdin_with_trailing_newline() {
        assert_success(&run(&["-d"], b"data:,piped%20in\n"), b"piped in");
    }

    #[test]
    fn output_file() {
        let path =
            std::env::temp_dir().join(format!("dataurl-test-decode-{}.bin", std::process::id()));
        assert_success(
            &run(
                &["-d", "-o", path.to_str().unwrap(), "data:;base64,+/8="],
                b"",
            ),
            b"",
        );
        assert_eq!(std::fs::read(&path).unwrap(), [0xFB, 0xFF]);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn empty_input() {
        assert_failure(&run(&["-d", ""], b""), "error: not a data URL\n");
    }

    #[test]
    fn invalid_base64() {
        assert_failure(
            &run(&["-d", "data:;base64,a"], b""),
            "error: data URL contains invalid base64\n",
        );
    }
}
