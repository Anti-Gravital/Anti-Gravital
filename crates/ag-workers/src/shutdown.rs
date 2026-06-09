//! Graceful shutdown signalling.
//!
//! A [`Shutdown`] is a cheap, clonable handle over a cancellation token. Worker pools
//! observe it to stop leasing, let in-flight jobs finish (up to a timeout) and exit
//! without losing leased jobs (on the durable backend an unacked job becomes leasable
//! again after its lease expires).

use tokio_util::sync::CancellationToken;

/// A clonable shutdown signal shared by the runtime and its worker pools.
#[derive(Debug, Clone, Default)]
pub struct Shutdown {
    token: CancellationToken,
}

impl Shutdown {
    /// Creates a fresh, un-triggered shutdown signal.
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    /// Triggers shutdown; all observers are notified.
    pub fn trigger(&self) {
        self.token.cancel();
    }

    /// Returns `true` if shutdown has been triggered.
    pub fn is_triggered(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Resolves when shutdown is triggered.
    pub async fn wait(&self) {
        self.token.cancelled().await
    }

    /// Returns the underlying token (for `select!` composition).
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn trigger_is_observed() {
        let s = Shutdown::new();
        assert!(!s.is_triggered());
        let s2 = s.clone();
        s.trigger();
        assert!(s2.is_triggered());
        s2.wait().await;
    }
}
