//! Serializes arbitrary data with fuzzer-chosen settings: it must parse back to the same data.

#![no_main]

use libfuzzer_sys::fuzz_target;

#[path = "../../tests/common/roundtrip.rs"]
mod roundtrip;

fuzz_target!(|input: &[u8]| {
    // The first bytes pick the settings, the rest is the data
    let (choices, data) = input.split_at(input.len().min(16));
    roundtrip::check_serialize(data, choices);
});
