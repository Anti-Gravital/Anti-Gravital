#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // El lexer nunca debe entrar en panico con texto UTF-8 arbitrario
        let _ = ag_dsl::lint(s);
    }
});
