//! Parses arbitrary input: it must never panic, and whatever parses must round-trip.

#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/common/roundtrip.rs"]
mod roundtrip;

fuzz_target!(|input: &[u8]| {
    roundtrip::check_parse(&String::from_utf8_lossy(input));
});
