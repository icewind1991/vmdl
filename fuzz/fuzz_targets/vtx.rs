#![no_main]
use libfuzzer_sys::fuzz_target;

fn fuzz(data: &[u8]) {
    let _ = vmdl::Vtx::read(data).ok();
}

fuzz_target!(|data: &[u8]| {fuzz(data)});
