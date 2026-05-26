//! TLS 1.3 layer with rustls.
//!
//! TLS operates at the connection level, not the request level, so this
//! layer is not integrated as Tower middleware. It is materialized as a
//! `TlsAcceptor` that wraps each accepted `TcpStream` before delegating
//! to the HTTP router.
//!
//! The commonly used public helper is `Shield::serve(listener, router)`
//! in the parent module, which detects whether the TLS layer is active
//! and dispatches to the correct transport.

use std::path::Path;
use std::sync::Arc;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;

use crate::config::TlsConfig;
use crate::error::AgError;

/// Builds a `TlsAcceptor` from the declarative configuration.
///
/// The cryptographic provider (`ring`) is injected explicitly into the
/// `ServerConfig`. It does not rely on rustls's global `CryptoProvider`,
/// which would be subject to a race between installation and read when
/// several process components touch TLS in parallel.
///
/// # Errors
///
/// Returns `AgError::Tls` if the files do not exist, cannot be parsed as
/// PEM, or do not contain at least one compatible certificate and
/// private key.
pub fn build_acceptor(config: &TlsConfig) -> Result<TlsAcceptor, AgError> {
    let cert_path = config
        .cert_path
        .as_deref()
        .ok_or_else(|| AgError::Tls("tls.cert_path is required when tls.enabled".to_owned()))?;
    let key_path = config
        .key_path
        .as_deref()
        .ok_or_else(|| AgError::Tls("tls.key_path is required when tls.enabled".to_owned()))?;

    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| AgError::Tls(format!("invalid TLS protocol versions: {e}")))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| AgError::Tls(format!("invalid server certificate: {e}")))?;

    Ok(TlsAcceptor::from(Arc::new(server_config)))
}

fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, AgError> {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(path)
        .map_err(|e| {
            AgError::Tls(format!(
                "cannot read certificates at {}: {e}",
                path.display()
            ))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AgError::Tls(format!("invalid PEM at {}: {e}", path.display())))?;
    if certs.is_empty() {
        return Err(AgError::Tls(format!(
            "no certificates found at {}",
            path.display()
        )));
    }
    Ok(certs)
}

fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, AgError> {
    PrivateKeyDer::from_pem_file(path)
        .map_err(|e| AgError::Tls(format!("invalid private key at {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn tmpfile(contents: &[u8]) -> std::path::PathBuf {
        let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("ag-core-tls-{pid}-{n}.pem"));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(contents).unwrap();
        path
    }

    fn assert_tls_error(cfg: &TlsConfig) {
        match build_acceptor(cfg) {
            Ok(_) => panic!("expected TLS error"),
            Err(err) => assert_eq!(err.code(), "tls_error"),
        }
    }

    #[test]
    fn missing_cert_path_errors() {
        let cfg = TlsConfig {
            enabled: true,
            cert_path: None,
            key_path: Some(std::path::PathBuf::from("/nope")),
        };
        assert_tls_error(&cfg);
    }

    #[test]
    fn missing_key_path_errors() {
        let cfg = TlsConfig {
            enabled: true,
            cert_path: Some(std::path::PathBuf::from("/nope")),
            key_path: None,
        };
        assert_tls_error(&cfg);
    }

    #[test]
    fn nonexistent_cert_file_errors() {
        let cfg = TlsConfig {
            enabled: true,
            cert_path: Some(std::path::PathBuf::from("/does/not/exist.pem")),
            key_path: Some(std::path::PathBuf::from("/does/not/exist.pem")),
        };
        assert_tls_error(&cfg);
    }

    #[test]
    fn empty_pem_errors() {
        let path = tmpfile(b"");
        let cfg = TlsConfig {
            enabled: true,
            cert_path: Some(path.clone()),
            key_path: Some(path),
        };
        assert_tls_error(&cfg);
    }
}
