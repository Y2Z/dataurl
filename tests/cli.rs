use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn data_file(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "tests", "_data_", name]
        .iter()
        .collect()
}

/// An empty directory of its own for each test, removed when dropped
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let path = std::env::temp_dir().join(format!("dataurl-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    fn file(&self, name: &str, contents: &[u8]) -> String {
        let path = self.0.join(name);
        std::fs::write(&path, contents).unwrap();
        path.to_str().unwrap().to_string()
    }

    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().to_string()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
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
    fn conflicting_options() {
        for (args, message) in [
            (
                &["-b", "-s", "x"][..],
                "'--base64' and '--shortest' can't be used together",
            ),
            (
                &["-d", "--size", "x"],
                "'--size' can't be used together with '--decode'",
            ),
            (
                &["-O", "dir", "x"],
                "'--output-dir' only works together with '--decode'",
            ),
            (
                &["-d", "-o", "file", "-O", "dir", "x"],
                "'--output-file' and '--output-dir' can't be used together",
            ),
            (&["-i", "-", "-i", "-"], "STDIN can only be read once"),
        ] {
            let output = run(args, b"");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.starts_with(&format!("error: {message}\n")),
                "{stderr}"
            );
            assert_eq!(output.status.code(), Some(2));
        }
    }

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
    fn shortest() {
        assert_success(&run(&["-s", "abc"], b""), b"data:,abc\n");
        assert_success(
            &run(&["--shortest"], &[0xFF; 30]),
            b"data:;base64,////////////////////////////////////////\n",
        );
    }

    #[test]
    fn minimal_escaping() {
        assert_success(&run(&["-m", "<a b='c'>"], b""), b"data:,%3Ca b='c'%3E\n");
        assert_success(
            &run(&["--minimal", "<a b='c'>"], b""),
            b"data:,%3Ca b='c'%3E\n",
        );
        assert_success(&run(&["<a b='c'>"], b""), b"data:,%3Ca%20b=%27c%27%3E\n");
    }

    #[test]
    fn size() {
        // "data:,a%20b" and "data:;base64,YSBi"
        assert_success(&run(&["--size", "a b"], b""), b"11\t17\n");
        // Other options count too
        assert_success(&run(&["--size", "-m", "-f", "x", "a b"], b""), b"11\t19\n");
    }

    #[test]
    fn size_of_files() {
        let dir = TempDir::new("size");
        let a = dir.file("a.txt", b"a b");
        let png = data_file("lizard.png");
        let png = png.to_str().unwrap();

        let output = run(&["--size", "-i", &a, "-i", png], b"");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines[0], format!("21\t27\t{a}"));

        let encoded = |args: &[&str]| run(args, b"").stdout.len() - 1;
        assert_eq!(
            lines[1],
            format!(
                "{}\t{}\t{png}",
                encoded(&["-i", png]),
                encoded(&["-b", "-i", png])
            )
        );
        assert_eq!(lines.len(), 2);
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
    fn several_input_files() {
        let dir = TempDir::new("several-inputs");
        let css = dir.file("style.css", b"a{}");
        let txt = dir.file("notes", b"x y");
        assert_success(
            &run(&["-i", &css, "-i", "-", "--input-file", &txt], b"<svg/>"),
            b"data:text/css,a{}\ndata:image/svg+xml,%3Csvg/%3E\ndata:,x%20y\n",
        );
    }

    #[test]
    fn several_input_files_with_options() {
        let dir = TempDir::new("several-inputs-options");
        let a = dir.file("a.txt", b"a");
        let b = dir.file("b.txt", b"b");
        assert_success(
            &run(&["-b", "-t", "x/y", "-f", "z", "-i", &a, "-i", &b], b""),
            b"data:x/y;base64,YQ==#z\ndata:x/y;base64,Yg==#z\n",
        );
    }

    #[test]
    fn unreadable_one_of_several_input_files() {
        let dir = TempDir::new("several-inputs-unreadable");
        let a = dir.file("a.txt", b"a");
        let output_file = dir.path("out.txt");
        let output = run(
            &["-i", &a, "-i", &dir.path("missing"), "-o", &output_file],
            b"",
        );
        assert!(!output.status.success());
        // What came before the error still gets written
        assert_eq!(std::fs::read(&output_file).unwrap(), b"data:text/plain,a\n");
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
    fn several_inputs_need_output_dir() {
        let dir = TempDir::new("decode-several-no-dir");
        let a = dir.file("a.txt", b"data:,a");
        let output = run(&["-d", "-i", &a, "-i", &a], b"");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.starts_with("error: decoding more than one input needs '--output-dir'\n"));
        assert_eq!(output.status.code(), Some(2));
    }

    #[test]
    fn output_dir() {
        let dir = TempDir::new("decode-output-dir");
        let png = std::fs::read(data_file("lizard.png")).unwrap();
        let png_url = format!(
            "data:image/png;base64,{}",
            run(&["-b", "-t", "x/x"], &png).stdout[16..]
                .iter()
                .map(|&b| b as char)
                .collect::<String>()
                .trim_end()
        );

        let inputs = [
            dir.file("logo.txt", png_url.as_bytes()),
            // Already has the right extension before ".txt"
            dir.file("icon.png.txt", png_url.as_bytes()),
            // Unknown media type, but the data has a PNG signature
            dir.file("other", png_url.replace("image/png", "x/x").as_bytes()),
            dir.file("page.url", b"data:text/html,<p>hi</p>\n"),
            dir.file("blob", b"data:application/x-unknown,%00%01"),
            dir.file("note", b"data:,plain"),
            // Same name as before, so it gets numbered
            dir.file("note.txt", b"data:,another"),
        ];
        let out = dir.path("out/nested");

        let mut args = vec!["-d", "--output-dir", &out];
        for input in &inputs {
            args.extend(["-i", input]);
        }
        let output = run(&args, b"");

        let expected = [
            ("logo.png", &png[..]),
            ("icon.png", &png),
            ("other.png", &png),
            ("page.html", b"<p>hi</p>"),
            ("blob.bin", b"\x00\x01"),
            ("note.txt", b"plain"),
            ("note-2.txt", b"another"),
        ];
        let mut expected_stdout = String::new();
        for (name, contents) in expected {
            let path = std::path::Path::new(&out).join(name);
            assert_eq!(std::fs::read(&path).unwrap(), contents, "{name}");
            expected_stdout.push_str(&format!("{}\n", path.display()));
        }
        assert_success(&output, expected_stdout.as_bytes());
    }

    #[test]
    fn output_dir_from_argument_and_stdin() {
        let dir = TempDir::new("decode-output-dir-stdin");
        let out = dir.path("out");
        let path = std::path::Path::new(&out).join("data.css");
        assert_success(
            &run(&["-d", "-O", &out, "data:text/css,a{}"], b""),
            format!("{}\n", path.display()).as_bytes(),
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"a{}");

        let path = std::path::Path::new(&out).join("data.svg");
        assert_success(
            &run(&["-d", "-O", &out], b"data:image/svg+xml,<svg/>"),
            format!("{}\n", path.display()).as_bytes(),
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"<svg/>");
    }

    #[test]
    fn invalid_one_of_several_inputs() {
        let dir = TempDir::new("decode-invalid-input");
        let bad = dir.file("bad.txt", b"data:x");
        let output = run(&["-d", "-O", &dir.path("out"), "-i", &bad], b"");
        assert_failure(
            &output,
            &format!("error: {bad}: data URL has no comma separating metadata from data\n"),
        );
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
