//! Declared environment-variable loading (RFC-0020 stage B).
//!
//! A project lists the variables it consumes as [`EnvDeclaration`]s. The
//! loader reads **only** those keys, validates their types and returns an
//! immutable [`LoadedEnv`]. Undeclared keys in the process environment are
//! never touched, so the declaration is the single source of truth for what
//! an application needs.
//!
//! # Startup contract
//!
//! Call [`EnvLoader::load_process`] in the composition root **before** any
//! TCP listener is opened and propagate the error as a non-zero exit:
//!
//! ```no_run
//! use ag_core::env::{EnvDeclaration, EnvLoader, EnvType};
//!
//! # fn main() -> Result<(), ag_core::env::EnvValidationError> {
//! let loader = EnvLoader::new(vec![
//!     EnvDeclaration::new("DATABASE_URL", EnvType::Url)
//!         .secret(true)
//!         .description_text("PostgreSQL connection string"),
//!     EnvDeclaration::new("PORT", EnvType::Int).with_default("8080"),
//! ]);
//! let env = loader.load_process()?;
//! let port = env.int("PORT").unwrap_or(8080);
//! let _ = (env.secret("DATABASE_URL"), port);
//! # Ok(())
//! # }
//! ```
//!
//! Every problem is collected into one [`EnvValidationError`] so the caller
//! can print all of them at once. Errors contain the variable name, the
//! expected type and the sources that were checked, but **never the value**,
//! including for variables that are not secrets.
//!
//! To bind this contract to configuration loading, use
//! [`crate::ShieldConfig::from_path_with_env`]: it resolves the declaration
//! before returning the config and the [`LoadedEnv`], so a composition root can
//! fail before binding a listener. Each raw value is taken out of the source
//! map and moved directly into [`Secret`] when declared secret; no unzeroized
//! clone is left behind.
//!
//! # Resolution order
//!
//! Stage B implements the first and last sources of RFC-0020 section 4.2:
//! process environment, then the declared default. The encrypted secrets
//! file and the `ag env` commands belong to later stages and are not
//! implemented here; the loader is deliberately a single pass so adding
//! sources does not change this contract. No external service is required.
//!
//! Existing crates keep their direct `std::env::var` calls: this module is
//! additive and migrates modules one pull request at a time (RFC-0020
//! section 5, PR 4).

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::AgError;
use crate::secret::Secret;

/// The type a declared variable must parse as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvType {
    /// Any UTF-8 string, including an empty one.
    String,
    /// A signed 64-bit integer.
    Int,
    /// `true`/`false` or `1`/`0`, case-insensitive for the words.
    Bool,
    /// An absolute URL with a scheme and a host.
    Url,
    /// A non-negative number of seconds with an optional `s`, `m`, `h` or
    /// `d` suffix (plain digits mean seconds).
    Duration,
    /// A non-empty path.
    Path,
}

impl fmt::Display for EnvType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Url => "url",
            Self::Duration => "duration",
            Self::Path => "path",
        };
        f.write_str(name)
    }
}

/// One entry of the declaration: a name, a type and its resolution policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvDeclaration {
    name: String,
    ty: EnvType,
    required: bool,
    secret: bool,
    default: Option<String>,
    doc: String,
}

impl EnvDeclaration {
    /// Declares a required, non-secret variable named `name`.
    pub fn new(name: impl Into<String>, ty: EnvType) -> Self {
        Self {
            name: name.into(),
            ty,
            required: true,
            secret: false,
            default: None,
            doc: String::new(),
        }
    }

    /// Variable name, exactly as read from the process environment.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Expected type used for validation.
    #[must_use]
    pub fn ty(&self) -> EnvType {
        self.ty
    }

    /// Whether the variable must resolve for loading to succeed.
    #[must_use]
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// Whether a resolved value is wrapped in [`Secret`].
    #[must_use]
    pub fn is_secret(&self) -> bool {
        self.secret
    }

    /// Declared fallback used when the process environment has no value.
    #[must_use]
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    /// Human-readable purpose of the variable.
    #[must_use]
    pub fn doc(&self) -> &str {
        &self.doc
    }

    /// Marks the variable required (`true`) or optional (`false`).
    /// Required is the default; an optional variable with no value is
    /// simply absent from [`LoadedEnv`].
    #[must_use]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Marks the value as secret. Secret values are delivered as
    /// `Secret<String>` and their raw bytes are preserved after validation.
    #[must_use]
    pub fn secret(mut self, secret: bool) -> Self {
        self.secret = secret;
        self
    }

    /// Sets a declared default. A default also satisfies `required`.
    #[must_use]
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the documentation string shown by future `ag env list`.
    #[must_use]
    pub fn description_text(mut self, doc: impl Into<String>) -> Self {
        self.doc = doc.into();
        self
    }
}

/// Why a declared variable could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvErrorReason {
    /// No value in any configured source and the variable was required.
    Missing,
    /// A value was found but did not parse as the declared type. The value
    /// itself is intentionally discarded and never stored here.
    InvalidSyntax,
    /// The process variable exists but is not valid UTF-8.
    NotUnicode,
}

impl fmt::Display for EnvErrorReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Missing => "missing (checked process environment and declared default)",
            Self::InvalidSyntax => "value failed type validation (value withheld)",
            Self::NotUnicode => "value is not valid Unicode (value withheld)",
        };
        f.write_str(text)
    }
}

/// A single resolution problem for one declared variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvError {
    name: String,
    expected: EnvType,
    reason: EnvErrorReason,
}

impl EnvError {
    /// Declared name of the variable.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Type the value was expected to parse as.
    #[must_use]
    pub fn expected(&self) -> EnvType {
        self.expected
    }

    /// Why resolution failed.
    #[must_use]
    pub fn reason(&self) -> &EnvErrorReason {
        &self.reason
    }
}

/// All problems found in one load, reported together before startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvValidationError {
    problems: Vec<EnvError>,
}

impl EnvValidationError {
    /// Problems in declaration order. Never empty for a real error.
    #[must_use]
    pub fn problems(&self) -> &[EnvError] {
        &self.problems
    }

    /// Number of problems found.
    #[must_use]
    pub fn len(&self) -> usize {
        self.problems.len()
    }

    /// Always `false` for an error produced by the loader.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.problems.is_empty()
    }
}

impl fmt::Display for EnvValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "environment validation failed ({} problem(s); values are never printed):",
            self.problems.len()
        )?;
        for problem in &self.problems {
            write!(
                f,
                "\n  - '{}': expected {}; {}",
                problem.name, problem.expected, problem.reason
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for EnvValidationError {}

impl From<EnvValidationError> for AgError {
    fn from(error: EnvValidationError) -> Self {
        Self::Config(error.to_string())
    }
}

/// A validated value returned by the loader.
#[derive(Debug)]
pub enum EnvValue {
    /// A string value.
    Text(String),
    /// An integer value.
    Int(i64),
    /// A boolean value.
    Bool(bool),
    /// A parsed absolute URL.
    Url(url::Url),
    /// A duration in seconds.
    Duration(Duration),
    /// A path value.
    Path(PathBuf),
    /// A secret string; only readable through [`Secret::expose`].
    Secret(Secret<String>),
}

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

    fn resolve(
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

enum RawValue {
    Missing,
    NotUnicode,
    Present(String),
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

#[cfg(test)]
mod tests {
    use super::*;

    const PROCESS_KEY: &str = "AG_CORE_ENVSECRETS_STAGE_B_PROCESS_KEY";

    struct EnvGuard;

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(PROCESS_KEY);
        }
    }

    #[test]
    fn reads_only_declared_keys() {
        let loader = EnvLoader::new(vec![EnvDeclaration::new("DECLARED", EnvType::String)]);
        let env = loader
            .load_from([
                ("DECLARED", "yes"),
                ("UNDECLARED", "ignored"),
                ("DATABASE_URL", "ignored"),
            ])
            .expect("declared variable resolves");
        assert_eq!(env.string("DECLARED"), Some("yes"));
        assert!(env.get("UNDECLARED").is_none());
        assert!(env.get("DATABASE_URL").is_none());
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn parses_all_supported_types() {
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("TEXT", EnvType::String),
            EnvDeclaration::new("COUNT", EnvType::Int),
            EnvDeclaration::new("FLAG", EnvType::Bool),
            EnvDeclaration::new("DB", EnvType::Url),
            EnvDeclaration::new("TIMEOUT", EnvType::Duration),
            EnvDeclaration::new("ROOT", EnvType::Path),
        ]);
        let env = loader
            .load_from([
                ("TEXT", "hello"),
                ("COUNT", "-42"),
                ("FLAG", "TRUE"),
                ("DB", "postgres://user@host:5432/db"),
                ("TIMEOUT", "2m"),
                ("ROOT", "/var/lib/ag"),
            ])
            .expect("all values are valid");
        assert_eq!(env.string("TEXT"), Some("hello"));
        assert_eq!(env.int("COUNT"), Some(-42));
        assert_eq!(env.boolean("FLAG"), Some(true));
        assert_eq!(
            env.url("DB").map(url::Url::as_str),
            Some("postgres://user@host:5432/db")
        );
        assert_eq!(env.duration("TIMEOUT"), Some(Duration::from_secs(120)));
        assert_eq!(env.path("ROOT"), Some(Path::new("/var/lib/ag")));
        // Accessors are type-safe: asking for the wrong type yields None.
        assert!(env.int("TEXT").is_none());
        assert!(env.string("COUNT").is_none());
    }

    #[test]
    fn duration_accepts_unit_suffixes_and_rejects_garbage() {
        let loader = EnvLoader::new(vec![EnvDeclaration::new("D", EnvType::Duration)]);
        for (raw, secs) in [
            ("0", 0),
            ("15", 15),
            ("15s", 15),
            ("3m", 180),
            ("2h", 7_200),
            ("1d", 86_400),
        ] {
            let env = loader.load_from([("D", raw)]).expect("valid duration");
            assert_eq!(env.duration("D"), Some(Duration::from_secs(secs)));
        }
        for raw in ["", "abc", "-1", "3x", "999999999999999999999999d"] {
            let err = loader.load_from([("D", raw)]).unwrap_err();
            assert_eq!(err.problems()[0].reason(), &EnvErrorReason::InvalidSyntax);
        }
    }

    #[test]
    fn collects_every_problem_before_startup_without_values() {
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("MISSING", EnvType::Url),
            EnvDeclaration::new("BAD_INT", EnvType::Int),
            EnvDeclaration::new("BAD_URL", EnvType::Url),
        ]);
        let error = loader
            .load_from([("BAD_INT", "not-a-number"), ("BAD_URL", "not-a-url")])
            .unwrap_err();
        assert_eq!(error.len(), 3);
        let names: Vec<&str> = error.problems().iter().map(EnvError::name).collect();
        assert_eq!(names, vec!["MISSING", "BAD_INT", "BAD_URL"]);
        assert_eq!(error.problems()[0].reason(), &EnvErrorReason::Missing);
        assert_eq!(error.problems()[1].expected(), EnvType::Int);
        assert_eq!(error.problems()[2].expected(), EnvType::Url);
        let rendered = error.to_string();
        assert!(!rendered.contains("not-a-number"));
        assert!(!rendered.contains("not-a-url"));
        assert!(rendered.contains("values are never printed"));
    }

    #[test]
    fn optional_absent_is_skipped_and_default_satisfies_required() {
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("OPTIONAL", EnvType::String).required(false),
            EnvDeclaration::new("PORT", EnvType::Int).with_default("8080"),
        ]);
        let env = loader
            .load_from(std::iter::empty::<(&str, &str)>())
            .expect("no required value missing");
        assert!(env.get("OPTIONAL").is_none());
        assert_eq!(env.int("PORT"), Some(8080));
        assert_eq!(env.len(), 1);
        assert!(!env.is_empty());

        let mut names = env.names();
        assert_eq!(names.next(), Some("PORT"));
        assert!(names.next().is_none());
    }

    #[test]
    fn invalid_default_is_reported_like_any_other_value() {
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("PORT", EnvType::Int).with_default("not-a-port")
        ]);
        let error = loader
            .load_from(std::iter::empty::<(&str, &str)>())
            .unwrap_err();
        assert_eq!(error.problems()[0].reason(), &EnvErrorReason::InvalidSyntax);
    }

    #[test]
    fn secret_values_are_redacted_and_exposed_explicitly() {
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("API_TOKEN", EnvType::String).secret(true)
        ]);
        let env = loader
            .load_from([("API_TOKEN", "  exact-token  ")])
            .expect("secret resolves");
        let secret = env.secret("API_TOKEN").expect("secret value");
        assert_eq!(secret.expose(), "  exact-token  ");
        assert_eq!(format!("{secret:?}"), "[redacted]");
        assert_eq!(format!("{secret}"), "[redacted]");
        assert!(!format!("{env:?}").contains("exact-token"));
    }

    #[test]
    fn invalid_secret_error_never_contains_the_value() {
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("SECRET_COUNT", EnvType::Int).secret(true)
        ]);
        let error = loader
            .load_from([("SECRET_COUNT", "super-secret-junk")])
            .unwrap_err();
        let rendered = error.to_string();
        assert!(!rendered.contains("super-secret-junk"));
        assert_eq!(error.problems()[0].name(), "SECRET_COUNT");
    }

    #[test]
    fn process_loader_reads_only_the_declared_key() {
        std::env::set_var(PROCESS_KEY, "5432");
        let _guard = EnvGuard;
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new(PROCESS_KEY, EnvType::Int),
            EnvDeclaration::new("AG_CORE_ENVSECRETS_UNDECLARED_PROCESS_KEY", EnvType::Int),
        ]);
        let env = loader.load_process().unwrap_err();
        // The second declaration was never set, so the load fails as a whole.
        assert_eq!(env.len(), 1);
        assert_eq!(
            env.problems()[0].name(),
            "AG_CORE_ENVSECRETS_UNDECLARED_PROCESS_KEY"
        );

        let loader = EnvLoader::new(vec![EnvDeclaration::new(PROCESS_KEY, EnvType::Int)]);
        let env = loader
            .load_process()
            .expect("declared process variable resolves");
        assert_eq!(env.int(PROCESS_KEY), Some(5432));
    }

    #[test]
    fn validation_error_converts_to_config_error() {
        let loader = EnvLoader::new(vec![EnvDeclaration::new("ABSENT", EnvType::String)]);
        let error: AgError = loader
            .load_from(std::iter::empty::<(&str, &str)>())
            .unwrap_err()
            .into();
        assert_eq!(error.code(), "config_error");
        assert!(error.to_string().contains("ABSENT"));
    }

    #[test]
    fn secret_source_is_taken_out_instead_of_cloned() {
        // Regression for wave-4 review observation O5. `resolve` must remove
        // each raw value from the source map rather than clone it, otherwise a
        // plain, unzeroized copy of a secret survives after the load.
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("TOKEN", EnvType::String).secret(true),
            EnvDeclaration::new("PORT", EnvType::Int),
        ]);
        let mut provided = HashMap::new();
        provided.insert(
            "TOKEN".to_owned(),
            RawValue::Present("s3cr3t-value".to_owned()),
        );
        provided.insert("PORT".to_owned(), RawValue::Present("8080".to_owned()));

        let env = loader.resolve(&mut provided).expect("values resolve");
        assert!(
            provided.is_empty(),
            "raw values must be taken out, not left as unzeroized clones"
        );
        assert_eq!(
            env.secret("TOKEN").expect("secret").expose(),
            "s3cr3t-value"
        );
        assert_eq!(env.int("PORT"), Some(8080));
    }

    #[test]
    fn typed_secret_preserves_exact_raw_bytes_without_leaking_them() {
        // Validation must not replace a secret's source with a parsed clone,
        // including for non-string declared types. The exact raw bytes move
        // into `Secret`; no parsed copy is left behind.
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("SECRET_URL", EnvType::Url).secret(true),
            EnvDeclaration::new("SECRET_COUNT", EnvType::Int).secret(true),
        ]);
        let env = loader
            .load_from([
                ("SECRET_URL", "  https://user:pw@db.example/  "),
                ("SECRET_COUNT", "  -42  "),
            ])
            .expect("typed secrets resolve");
        let url = env.secret("SECRET_URL").expect("url secret");
        let count = env.secret("SECRET_COUNT").expect("int secret");
        assert_eq!(url.expose(), "  https://user:pw@db.example/  ");
        assert_eq!(count.expose(), "  -42  ");
        let rendered = format!("{env:?}");
        assert!(!rendered.contains("pw@db"));
        assert!(!rendered.contains("-42"));
        assert!(rendered.contains("[redacted]"));
    }

    #[test]
    fn fail_closed_load_never_prints_a_value() {
        // A required secret that is missing and a malformed non-secret are
        // both reported in one error, and neither value nor source reaches the
        // rendered message.
        let loader = EnvLoader::new(vec![
            EnvDeclaration::new("NEVER_SET_DURING_TEST", EnvType::String).secret(true),
            EnvDeclaration::new("BAD", EnvType::Int),
        ]);
        let error = loader
            .load_from([("BAD", "top-secret-non-numeric")])
            .unwrap_err();
        assert_eq!(error.len(), 2);
        assert_eq!(error.problems()[0].reason(), &EnvErrorReason::Missing);
        assert_eq!(error.problems()[1].reason(), &EnvErrorReason::InvalidSyntax);
        let rendered = error.to_string();
        assert!(!rendered.contains("top-secret-non-numeric"));
        assert!(rendered.contains("NEVER_SET_DURING_TEST"));
        assert!(rendered.contains("BAD"));
        let as_config: AgError = error.into();
        assert!(!as_config.to_string().contains("top-secret-non-numeric"));
    }
}
