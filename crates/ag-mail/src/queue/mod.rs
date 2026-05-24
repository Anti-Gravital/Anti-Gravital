//! Cola asincrona de correo con reintentos y backoff exponencial.
//!
//! `MailQueue` es el trait que toda implementacion de cola debe satisfacer.
//! `InMemoryQueue` es el backend por defecto: un canal Tokio con un worker
//! que procesa los mensajes aplicando backoff exponencial en caso de fallo.
//!
//! La cola no garantiza durabilidad entre reinicios del proceso. Para
//! durabilidad, usar `queue-persistent` (feature opcional, Etapa futura)
//! que persiste los jobs en `ag-data`.

#[cfg(feature = "queue-persistent")]
pub mod store;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{error::AgMailError, message::Email, metrics, sender::MailSender};

/// Configuracion del comportamiento de reintentos.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Numero maximo de reintentos (sin contar el intento inicial).
    pub max_retries: u32,
    /// Delay base para backoff exponencial.
    pub base_delay: Duration,
    /// Multiplicador de backoff (se eleva al numero de reintento).
    pub backoff_factor: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(2),
            backoff_factor: 2,
        }
    }
}

impl RetryPolicy {
    /// Calcula el delay para el reintento `n` (0-based).
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let multiplier = self.backoff_factor.saturating_pow(attempt) as u64;
        self.base_delay.saturating_mul(multiplier as u32)
    }
}

/// Abstraccion sobre una cola de correo.
#[async_trait]
pub trait MailQueue: Send + Sync {
    /// Encola un correo para envio asincrono.
    ///
    /// Retorna `Ok(())` cuando el correo ha sido aceptado en la cola.
    /// El envio real ocurre en background.
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError>;
}

// Mensaje interno del canal.
struct QueueItem {
    email: Email,
    attempt: u32,
}

/// Cola en memoria respaldada por un canal Tokio y un worker asincrono.
///
/// El worker se lanza como una tarea de Tokio al construir la cola y
/// termina cuando todos los senders del canal se desconectan (i.e., cuando
/// la cola se descarta).
pub struct InMemoryQueue {
    tx: mpsc::Sender<QueueItem>,
}

impl InMemoryQueue {
    /// Crea la cola con un sender y politica de reintentos dada.
    ///
    /// `capacity` es el numero maximo de mensajes pendientes en el canal
    /// antes de que `enqueue` bloquee al caller (backpressure).
    pub fn new(sender: Arc<dyn MailSender>, policy: RetryPolicy, capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        tokio::spawn(worker(rx, sender, policy));
        Self { tx }
    }
}

#[async_trait]
impl MailQueue for InMemoryQueue {
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError> {
        self.tx
            .send(QueueItem { email, attempt: 0 })
            .await
            .map_err(|_| AgMailError::SendExhausted("cola cerrada".to_owned()))
    }
}

// Worker que consume el canal y reencola con backoff en caso de fallo.
async fn worker(
    mut rx: mpsc::Receiver<QueueItem>,
    sender: Arc<dyn MailSender>,
    policy: RetryPolicy,
) {
    // Necesitamos reencolar sin el rx consumido, usamos un canal secundario
    // para reintentos.
    let (retry_tx, mut retry_rx) = mpsc::channel::<QueueItem>(64);

    loop {
        // Selecciona el proximo item, priorizando los reintentos sobre los nuevos.
        let item = tokio::select! {
            biased;
            Some(item) = retry_rx.recv() => item,
            Some(item) = rx.recv() => item,
            else => break,
        };

        let provider = sender.name();
        let start = std::time::Instant::now();

        match sender.send(&item.email).await {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f64();
                info!(provider, "correo enviado");
                metrics::record_send(provider, true);
                metrics::record_send_latency(provider, elapsed);
            }
            Err(e) => {
                metrics::record_send(provider, false);

                if item.attempt < policy.max_retries {
                    let delay = policy.delay_for(item.attempt);
                    warn!(
                        provider,
                        attempt = item.attempt + 1,
                        max = policy.max_retries,
                        delay_ms = delay.as_millis(),
                        error = %e,
                        "fallo de envio, reintentando"
                    );
                    metrics::record_retry(provider);

                    let retry_tx = retry_tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(delay).await;
                        let _ = retry_tx
                            .send(QueueItem {
                                email: item.email,
                                attempt: item.attempt + 1,
                            })
                            .await;
                    });
                } else {
                    error!(
                        provider,
                        attempts = item.attempt + 1,
                        error = %e,
                        "correo descartado tras agotar reintentos"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Address, EmailBuilder};
    use crate::sender::test_utils::NullSender;

    fn test_email() -> Email {
        EmailBuilder::new()
            .from(Address::new("f@f.com"))
            .to(Address::new("t@t.com"))
            .subject("test")
            .text_body("body")
            .build()
            .unwrap()
    }

    #[test]
    fn retry_policy_delay_grows_exponentially() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            backoff_factor: 2,
        };
        assert_eq!(policy.delay_for(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for(2), Duration::from_millis(400));
    }

    #[tokio::test]
    async fn enqueue_accepts_email() {
        let sender = Arc::new(NullSender::new());
        let queue = InMemoryQueue::new(sender.clone(), RetryPolicy::default(), 16);
        queue.enqueue(test_email()).await.unwrap();
        // Pequena espera para que el worker procese.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(sender.emails_sent(), 1);
    }

    #[tokio::test]
    async fn enqueue_multiple_emails() {
        let sender = Arc::new(NullSender::new());
        let queue = InMemoryQueue::new(sender.clone(), RetryPolicy::default(), 16);
        for _ in 0..5 {
            queue.enqueue(test_email()).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(sender.emails_sent(), 5);
    }
}
