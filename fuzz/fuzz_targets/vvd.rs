#![no_main]
use libfuzzer_sys::fuzz_target;

fn fuzz(data: &[u8]) {
    let _ = vmdl::Vvd::read(data).ok();
}

fuzz_target!(|data: &[u8]| {fuzz(data)});
