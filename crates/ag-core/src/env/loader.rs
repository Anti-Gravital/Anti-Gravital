//! Environment loader and value parsing (RFC-0020 stage B).
//!
//! Moved from `env.rs` without changing resolution order, validation or the
//! zeroization behaviour of secret values.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{
    EnvDeclaration, EnvError, EnvErrorReason, EnvType, EnvValidationError, EnvValue, RawValue,
};
use crate::secret::Secret;

/// Immutable result of a successful load.
#[derive(Debug)]
pub struct LoadedEnv {
    values: HashMap<String, EnvValue>,
}

impl LoadedEnv {
    /// Returns the value for `name`, or `None` if it was optional and
    /// unresolved.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&EnvValue> {
        self.values.get(name)
    }

    /// Returns a string value, or `None` when absent or of another type.
    #[must_use]
    pub fn string(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(EnvValue::Text(value)) => Some(value),
            _ => None,
        }
    }

    /// Returns an integer value, or `None` when absent or of another type.
    #[must_use]
    pub fn int(&self, name: &str) -> Option<i64> {
        match self.get(name) {
            Some(EnvValue::Int(value)) => Some(*value),
            _ => None,
        }
    }

    /// Returns a boolean value, or `None` when absent or of another type.
    #[must_use]
    pub fn boolean(&self, name: &str) -> Option<bool> {
        match self.get(name) {
            Some(EnvValue::Bool(value)) => Some(*value),
            _ => None,
        }
    }

    /// Returns a URL value, or `None` when absent or of another type.
    #[must_use]
    pub fn url(&self, name: &str) -> Option<&url::Url> {
        match self.get(name) {
            Some(EnvValue::Url(value)) => Some(value),
            _ => None,
        }
    }

    /// Returns a duration value, or `None` when absent or of another type.
    #[must_use]
    pub fn duration(&self, name: &str) -> Option<Duration> {
        match self.get(name) {
            Some(EnvValue::Duration(value)) => Some(*value),
            _ => None,
        }
    }

    /// Returns a path value, or `None` when absent or of another type.
    #[must_use]
    pub fn path(&self, name: &str) -> Option<&Path> {
        match self.get(name) {
            Some(EnvValue::Path(value)) => Some(value.as_path()),
            _ => None,
        }
    }

    /// Returns a secret value, or `None` when absent or not declared secret.
    #[must_use]
    pub fn secret(&self, name: &str) -> Option<&Secret<String>> {
        match self.get(name) {
            Some(EnvValue::Secret(value)) => Some(value),
            _ => None,
        }
    }

    /// Number of resolved values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether no value was resolved.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Names of the resolved variables, in unspecified order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

/// A reusable declaration set used to resolve an environment.
#[derive(Debug, Clone)]
pub struct EnvLoader {
    declarations: Vec<EnvDeclaration>,
}

impl EnvLoader {
    /// Builds a loader from `declarations`. Names must be unique; if one is
    /// repeated, the first occurrence is resolved and later duplicates are
    /// ignored (they must not consume the raw value a second time).
    pub fn new(declarations: Vec<EnvDeclaration>) -> Self {
        Self { declarations }
    }

    /// The declarations this loader honors.
    #[must_use]
    pub fn declarations(&self) -> &[EnvDeclaration] {
        &self.declarations
    }

    /// Resolves from the process environment, reading only declared keys.
    ///
    /// Non-UTF-8 values are reported instead of panicking. No listener should
    /// be opened until this returns `Ok`.
    ///
    /// # Errors
    ///
    /// Returns every problem found in a single [`EnvValidationError`].
    pub fn load_process(&self) -> Result<LoadedEnv, EnvValidationError> {
        let mut provided = HashMap::with_capacity(self.declarations.len());
        for declaration in &self.declarations {
            let raw = match std::env::var_os(&declaration.name) {
                None => RawValue::Missing,
                Some(os) => match os.into_string() {
                    Ok(value) => RawValue::Present(value),
                    Err(_) => RawValue::NotUnicode,
                },
            };
            provided.insert(declaration.name.clone(), raw);
        }
        self.resolve(&mut provided)
    }

    /// Resolves from an injected set of `(key, value)` pairs.
    ///
    /// Useful for tests and for composition roots that obtain variables from
    /// another source. Keys outside the declaration are ignored, so this is
    /// safe to call with a full environment map. With duplicate keys, the
    /// last one wins.
    ///
    /// # Errors
    ///
    /// Returns every problem found in a single [`EnvValidationError`].
    pub fn load_from<I, K, V>(&self, vars: I) -> Result<LoadedEnv, EnvValidationError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let declared: HashSet<&str> = self
            .declarations
            .iter()
            .map(|declaration| declaration.name.as_str())
            .collect();
        let mut provided = HashMap::with_capacity(self.declarations.len());
        for (key, value) in vars {
            let key = key.as_ref();
            if declared.contains(key) {
                provided.insert(key.to_owned(), RawValue::Present(value.as_ref().to_owned()));
            }
        }
        self.resolve(&mut provided)
    }

    pub(super) fn resolve(
        &self,
        provided: &mut HashMap<String, RawValue>,
    ) -> Result<LoadedEnv, EnvValidationError> {
        let mut values = HashMap::with_capacity(self.declarations.len());
        let mut problems = Vec::new();
        // Declaration names must be unique. If a caller repeats one, only the
        // first occurrence consumes the raw value; later duplicates are ignored
        // instead of reporting a spurious "missing" problem.
        let mut seen = HashSet::with_capacity(self.declarations.len());

        for declaration in &self.declarations {
            if !seen.insert(declaration.name.as_str()) {
                continue;
            }
            // Take the raw value out of the map instead of cloning it. A secret
            // moved into `Secret` is zeroized on drop; a clone left behind in
            // `provided` would be dropped as a plain, unzeroized String
            // (wave-4 review observation O5).
            let raw = provided
                .remove(&declaration.name)
                .unwrap_or(RawValue::Missing);
            let candidate = match raw {
                RawValue::Present(value) => Some(value),
                RawValue::NotUnicode => {
                    problems.push(problem(declaration, EnvErrorReason::NotUnicode));
                    continue;
                }
                RawValue::Missing => declaration.default.clone(),
            };

            let Some(candidate) = candidate else {
                if declaration.required {
                    problems.push(problem(declaration, EnvErrorReason::Missing));
                }
                continue;
            };

            // Validate without allocating a copy of `candidate`. Parsing first
            // and discarding the parsed value would leave a second, unzeroized
            // copy of secret material (for example a secret declared as a
            // string or URL). The single owned String now moves straight into
            // `Secret`; non-secret values are parsed once into their final type.
            if !is_valid(declaration.ty, &candidate) {
                problems.push(problem(declaration, EnvErrorReason::InvalidSyntax));
                continue;
            }
            let value = if declaration.secret {
                EnvValue::Secret(Secret::new(candidate))
            } else {
                parse_value(declaration.ty, &candidate)
                    .expect("is_valid guarantees the value parses")
            };
            values.insert(declaration.name.clone(), value);
        }

        if problems.is_empty() {
            Ok(LoadedEnv { values })
        } else {
            Err(EnvValidationError { problems })
        }
    }
}

fn problem(declaration: &EnvDeclaration, reason: EnvErrorReason) -> EnvError {
    EnvError {
        name: declaration.name.clone(),
        expected: declaration.ty,
        reason,
    }
}

/// Checks that `raw` parses as `ty` without allocating a copy of `raw`.
///
/// Kept separate from [`parse_value`] so validating a secret never leaves a
/// second, unzeroized copy of the material in a discarded parsed value. The two
/// functions must agree: `is_valid` is a pure pre-check, `parse_value` builds
/// the final owned value only after validation succeeded.
fn is_valid(ty: EnvType, raw: &str) -> bool {
    match ty {
        EnvType::String => true,
        EnvType::Int => raw.trim().parse::<i64>().is_ok(),
        EnvType::Bool => parse_bool(raw.trim()).is_some(),
        EnvType::Url => match url::Url::parse(raw.trim()) {
            Ok(url) => url.has_host(),
            Err(_) => false,
        },
        EnvType::Duration => parse_duration(raw.trim()).is_some(),
        EnvType::Path => !raw.trim().is_empty(),
    }
}

fn parse_value(ty: EnvType, raw: &str) -> Option<EnvValue> {
    match ty {
        EnvType::String => Some(EnvValue::Text(raw.to_owned())),
        EnvType::Int => raw.trim().parse::<i64>().ok().map(EnvValue::Int),
        EnvType::Bool => parse_bool(raw.trim()).map(EnvValue::Bool),
        EnvType::Url => {
            let url = url::Url::parse(raw.trim()).ok()?;
            url.has_host().then_some(EnvValue::Url(url))
        }
        EnvType::Duration => parse_duration(raw.trim()).map(EnvValue::Duration),
        EnvType::Path => {
            if raw.trim().is_empty() {
                None
            } else {
                Some(EnvValue::Path(PathBuf::from(raw)))
            }
        }
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    if raw.eq_ignore_ascii_case("true") || raw == "1" {
        Some(true)
    } else if raw.eq_ignore_ascii_case("false") || raw == "0" {
        Some(false)
    } else {
        None
    }
}

fn parse_duration(raw: &str) -> Option<Duration> {
    if raw.is_empty() {
        return None;
    }
    let (digits, multiplier) = match raw.as_bytes()[raw.len() - 1] {
        b's' => (&raw[..raw.len() - 1], 1_u64),
        b'm' => (&raw[..raw.len() - 1], 60),
        b'h' => (&raw[..raw.len() - 1], 3_600),
        b'd' => (&raw[..raw.len() - 1], 86_400),
        _ => (raw, 1),
    };
    let seconds = digits.parse::<u64>().ok()?;
    seconds.checked_mul(multiplier).map(Duration::from_secs)
}
