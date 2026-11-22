//! `ag config` command: flag path and interactive form share one executor.

use crate::cli::ConfigCommands;
use crate::config::{
    config_path, current_values, execute_config, ConfigChange, ConfigKey, ConfigOutcome,
    ConfigRequest, CONFIG_FILE,
};
use crate::interactive::Session;

/// Why `ag config` could not complete, mapped to an exit code by `main`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConfigCommandError {
    /// Incomplete or invalid command line (exit code 2).
    Usage(String),
    /// Reading, validating or writing the file failed (exit code 1).
    Failed(String),
}

/// Entry point of `ag config`.
pub(crate) fn cmd_config(
    command: Option<ConfigCommands>,
    no_interactive: bool,
) -> Result<(), ConfigCommandError> {
    let path = config_path();
    let (request, from_form) = match command {
        Some(ConfigCommands::Get { key }) => (
            ConfigRequest::Get {
                key: parse_key(&key)?,
            },
            false,
        ),
        Some(ConfigCommands::Set { key, value }) => (
            ConfigRequest::Set {
                changes: vec![ConfigChange {
                    key: parse_key(&key)?,
                    value,
                }],
            },
            false,
        ),
        Some(ConfigCommands::Validate) => (ConfigRequest::Validate, false),
        None => {
            let session = Session::from_environment(no_interactive);
            if session.is_interactive() && cfg!(feature = "tui") {
                let values = current_values(&path).map_err(ConfigCommandError::Failed)?;
                match ask_config(&values, &session)? {
                    Some(request) => (request, true),
                    None => {
                        println!("Cancelled. {CONFIG_FILE} was not changed.");
                        return Ok(());
                    }
                }
            } else {
                return Err(ConfigCommandError::Usage(
                    "`ag config` needs a terminal for the form.\n\
                     Non-interactive usage: ag config get KEY | set KEY VALUE | validate"
                        .to_owned(),
                ));
            }
        }
    };

    let outcome = execute_config(&request, &path).map_err(ConfigCommandError::Failed)?;
    match outcome {
        ConfigOutcome::Value(value) => println!("{value}"),
        ConfigOutcome::Valid => println!("{CONFIG_FILE} is valid."),
        ConfigOutcome::Written { changes } => {
            println!("Updated {CONFIG_FILE} ({changes} change(s)).");
            if from_form {
                if let ConfigRequest::Set { changes } = &request {
                    println!("Equivalent non-interactive command:");
                    for change in changes {
                        println!("  {}", change.equivalent_command());
                    }
                }
            }
        }
    }
    Ok(())
}

/// Parses a key with a helpful error naming every accepted key.
fn parse_key(value: &str) -> Result<ConfigKey, ConfigCommandError> {
    ConfigKey::parse(value).ok_or_else(|| {
        let keys: Vec<&str> = ConfigKey::ALL.iter().map(|key| key.as_str()).collect();
        ConfigCommandError::Usage(format!(
            "unknown configuration key '{value}'.\nusage: ag config get KEY | set KEY VALUE\nkeys: {}",
            keys.join(", ")
        ))
    })
}

#[cfg(feature = "tui")]
fn ask_config(
    values: &[(ConfigKey, String)],
    session: &Session,
) -> Result<Option<ConfigRequest>, ConfigCommandError> {
    crate::tui::ask_config(values, session).map_err(ConfigCommandError::Failed)
}

#[cfg(not(feature = "tui"))]
fn ask_config(
    _values: &[(ConfigKey, String)],
    _session: &Session,
) -> Result<Option<ConfigRequest>, ConfigCommandError> {
    Err(ConfigCommandError::Failed(
        "this build of ag has no interactive interface".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn flags_produce_the_same_request_shape_as_the_form() {
        let cli =
            Cli::try_parse_from(["ag", "config", "set", "bind", "127.0.0.1:7000"]).expect("parse");
        let Some(crate::cli::Commands::Config {
            command: Some(ConfigCommands::Set { key, value }),
            ..
        }) = cli.command
        else {
            panic!("expected config set");
        };
        let flag_request = ConfigRequest::Set {
            changes: vec![ConfigChange {
                key: parse_key(&key).expect("key"),
                value,
            }],
        };
        let form_request = ConfigRequest::Set {
            changes: vec![ConfigChange {
                key: ConfigKey::Bind,
                value: "127.0.0.1:7000".into(),
            }],
        };
        assert_eq!(flag_request, form_request);
    }

    #[test]
    fn an_unknown_key_is_a_usage_error_naming_the_keys() {
        let error = parse_key("not.a.key").expect_err("unknown key");
        let ConfigCommandError::Usage(message) = error else {
            panic!("expected a usage error");
        };
        assert!(message.contains("bind"));
        assert!(message.contains("tls.key_path"));
    }
}
