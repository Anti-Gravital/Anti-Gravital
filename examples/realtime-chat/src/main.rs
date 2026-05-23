//! Realtime chat example using ag-realtime and SSE.

mod handlers;
use ag_realtime::AgRealtime;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub realtime: Arc<AgRealtime>,
}

fn main() {}
