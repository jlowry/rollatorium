#![no_main]

use libfuzzer_sys::fuzz_target;
use rollatorium::{eval, parse};

fuzz_target!(|data: &[u8]| {
    let expr = std::string::String::from_utf8_lossy(data);
    if let Ok(ast) = parse(&expr) {
        let _ = eval(&ast);
    }
});
