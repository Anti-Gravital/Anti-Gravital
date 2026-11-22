//! Secret wrapper with redacted formatting and zeroization on drop.
//!
//! Defined by `docs/rfc/RFC-0020-environment-and-secrets.md` section 4.3.
//! A [`Secret<T>`] is the only supported way to keep confidential
//! configuration material in memory. It deliberately implements neither
//! [`serde::Serialize`] nor a formatting trait that could reveal the inner
//! value: reading it always requires an explicit [`Secret::expose`] call,
//! which makes every use of a secret searchable during review.

use std::fmt;

use zeroize::Zeroize;

/// A confidential value that never reveals itself through formatting or
/// serialization.
///
/// `Debug` and `Display` both print the literal `[redacted]`. There is no
/// `Serialize` implementation, so a `Secret<T>` cannot be embedded in a
/// log record, an HTTP response or a configuration dump by accident. It also
/// implements no comparison traits, so a secret is never compared by value.
/// When the value is dropped its bytes are overwritten through
/// [`zeroize::Zeroize`].
///
/// # Example
///
/// ```
/// use ag_core::secret::Secret;
///
/// let token = Secret::new("s3cr3t".to_owned());
/// assert_eq!(format!("{token:?}"), "[redacted]");
/// assert_eq!(format!("{token}"), "[redacted]");
/// assert_eq!(token.expose(), "s3cr3t");
/// ```
///
/// Trying to serialize a secret is a compile error:
///
/// ```compile_fail
/// use ag_core::secret::Secret;
///
/// let token = Secret::new("s3cr3t".to_owned());
/// let _ = serde_json::to_string(&token);
/// ```
pub struct Secret<T: Zeroize> {
    inner: T,
}

impl<T: Zeroize> Secret<T> {
    /// Wraps `value` in a secret.
    ///
    /// The caller is responsible for not duplicating the value elsewhere;
    /// zeroization only covers the bytes owned by this instance.
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }

    /// Borrows the confidential value.
    ///
    /// Every call is intentional and greppable. No mutable accessor is
    /// provided, so the wrapped value cannot be replaced or extracted.
    #[must_use]
    pub fn expose(&self) -> &T {
        &self.inner
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn debug_and_display_are_redacted() {
        let secret = Secret::new("hunter2".to_owned());
        assert_eq!(format!("{secret:?}"), "[redacted]");
        assert_eq!(format!("{secret}"), "[redacted]");
        assert_eq!(format!("{secret:#?}"), "[redacted]");
        // Redaction also wins inside derived containers.
        let secrets = vec![secret];
        assert!(!format!("{secrets:?}").contains("hunter2"));
    }

    #[test]
    fn expose_returns_the_exact_value() {
        let secret = Secret::new(vec![1_u8, 2, 3]);
        assert_eq!(secret.expose(), &[1, 2, 3]);
        assert_eq!(secret.expose().len(), 3);
    }

    #[test]
    fn drop_zeroizes_the_inner_value() {
        struct Probe {
            value: u64,
            zeroized: Rc<Cell<bool>>,
        }

        impl Zeroize for Probe {
            fn zeroize(&mut self) {
                self.value = 0;
                self.zeroized.set(true);
            }
        }

        let observed = Rc::new(Cell::new(false));
        {
            let secret = Secret::new(Probe {
                value: 42,
                zeroized: Rc::clone(&observed),
            });
            assert_eq!(secret.expose().value, 42);
            assert!(!observed.get());
        }
        assert!(observed.get(), "Drop must call Zeroize::zeroize");
    }

    #[test]
    fn string_secret_is_redacted_after_a_format_arg_roundtrip() {
        let secret = Secret::new("token-value".to_owned());
        let rendered = format!("authorization={secret}");
        assert_eq!(rendered, "authorization=[redacted]");
        assert!(!rendered.contains("token-value"));
    }
}
