#![no_main]
//! Fuzzes the ag-cache native RESP2 command parser.
//!
//! Arbitrary bytes are fed through `read_command`, which must never panic,
//! overflow, or allocate unboundedly (a tiny header must not trigger a huge
//! allocation). See the bulk-string / multibulk length guards in `resp.rs`.

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;
use tokio::io::BufReader;

fuzz_target!(|data: &[u8]| {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("current-thread runtime");
    rt.block_on(async {
        let mut reader = BufReader::new(Cursor::new(data.to_vec()));
        let _ = ag_cache::server::resp::read_command(&mut reader).await;
    });
});
