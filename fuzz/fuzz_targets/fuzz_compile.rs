#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // El pipeline completo (lex+parse+semantic+codegen) no debe entrar en panico
        let diags = ag_dsl::lint(s);
        if diags.iter().all(|d| !d.is_error()) {
            if let Ok(schema) = ag_dsl::compile(s) {
                let _ = ag_dsl::generate(&schema);
            }
        }
    }
});
