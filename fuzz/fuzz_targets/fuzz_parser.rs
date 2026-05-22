#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // El parser nunca debe entrar en panico con entrada arbitraria
        let _ = ag_dsl::lint(s);
    }
});
