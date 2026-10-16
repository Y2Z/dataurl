//! Randomized round-trip tests, using the same checks as the fuzz targets (fuzz/).
//! They're deterministic: every run uses the same inputs, so failures can be reproduced.
//! Set DATAURL_ROUNDTRIP_ITERATIONS to run more of them.

mod common;

use common::roundtrip::{check_parse, check_serialize};

fn iterations() -> u64 {
    std::env::var("DATAURL_ROUNDTRIP_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5_000)
}

/// Deterministic pseudo-random numbers (xorshift)
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn bytes(&mut self, max_length: usize) -> Vec<u8> {
        let length = self.below(max_length + 1);
        (0..length).map(|_| self.next() as u8).collect()
    }
}

/// Pieces that data URLs are made of, so that random inputs are likely to hit interesting cases
const PIECES: &[&str] = &[
    "data:",
    "DATA:",
    ",",
    ";",
    "base64",
    "BASE64",
    ";base64",
    "; base64",
    "charset=",
    "=",
    "\"",
    "\\",
    "%",
    "%2",
    "%2C",
    "%25",
    "%3B",
    "%ZZ",
    "#",
    "?",
    "/",
    "//",
    "//host/",
    "/../",
    "/./",
    "text/plain",
    "text/html",
    "image/svg+xml",
    "x/x",
    " ",
    "  ",
    "\t",
    "\n",
    "\r",
    "\u{c}",
    "\0",
    "\u{7f}",
    "é",
    "†",
    "💩",
    "a",
    "Z",
    "0",
    "+",
    "=",
    "==",
    "QQ",
    "QUJD",
    "WA",
    "SGVsbG8",
    "<svg>",
    "&amp;",
    "'",
    "(",
    ")",
    "[::1]",
    ":80",
    "@",
    "utf-8",
    "UTF-8",
    "gbk",
];

fn random_data_url(random: &mut Random) -> String {
    let mut input = String::new();
    if random.below(10) != 0 {
        input.push_str("data:");
    }
    for _ in 0..random.below(12) {
        input.push_str(PIECES[random.below(PIECES.len())]);
    }
    input
}

#[test]
fn random_inputs_parse_and_round_trip() {
    let mut random = Random(0x5EED_DA7A_0001);
    for _ in 0..iterations() {
        check_parse(&random_data_url(&mut random));
    }
}

#[test]
fn random_bytes_parse_without_panicking() {
    let mut random = Random(0x5EED_DA7A_0002);
    for _ in 0..iterations() {
        let mut bytes = b"data:".to_vec();
        bytes.extend(random.bytes(64));
        check_parse(&String::from_utf8_lossy(&bytes));
    }
}

#[test]
fn random_data_serializes_and_round_trips() {
    let mut random = Random(0x5EED_DA7A_0003);
    for _ in 0..iterations() {
        let data = random.bytes(200);
        let choices = random.bytes(16);
        check_serialize(&data, &choices);
    }
}

#[test]
fn known_tricky_inputs() {
    for input in [
        "data:",
        "data:,",
        "data:;base64,",
        "data:;base64;base64,WA",
        "data:x/x;base64;charset=x;base64,WA",
        "data:text/plain;a=\";base64\",x",
        "data:;charset=\"\\\"\",x",
        "data:,a%",
        "data:,%%%",
        "data:,  #  ",
        "data://test/a/../,X",
        "data://[::1]:80/,X",
        "data:/x/./,X",
        "data:\u{0}\t\n,x",
    ] {
        check_parse(input);
    }
}
