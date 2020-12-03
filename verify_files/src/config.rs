//! Definitions for the `verifiers.toml` configuration file.

// Standard library imports
use std::collections::HashMap;
use std::fmt::Write;

// 3rd-party crate imports
use log::warn;
use anyhow::{anyhow, Context, Result}; // It's an internal API, so no need for thiserror yet.
use serde_derive::{Serialize, Deserialize};
use validator::{Validate, ValidationError, ValidationErrors};

/// Helper to reverse Serde's usual behaviour for a `bool` with `default`
fn bool_true_default() -> bool { true }

/// Wrapper to compact the repeated boilerplate of attaching messages to a custom
/// validation failure
macro_rules! fail_valid {
    ($code:expr, $msg:expr) => {
        let mut err = ValidationError::new($code);
        err.message = Some($msg.into());
        return Err(err);
    }
}

/// Validator to verify structural correctness of extension definitions
fn validate_exts(input: &OneOrList<String>) -> ::std::result::Result<(), ValidationError> {
    // TODO: Report this false lint triggering as a bug
    if match *input {
        One(ref x) => x.is_empty(),
        List(ref x) => x.is_empty() || x.iter().any(String::is_empty),
    } {
        fail_valid!("empty_ext", "Extensions may not be empty strings");
    }

    // TODO: Once I no longer need to be cross-compatible with Python, omit the period from the
    // file, since Rust handles extension splitting differently.
    if match *input {
        One(ref x) => !x.starts_with('.'),
        List(ref x) => !x.iter().all(|y| y.starts_with('.')),
    } {
        fail_valid!("no_period_ext", "Extensions must start with a period");
    }


    Ok(())
}

/// Validator to verify that no header definitions are empty strings
fn validate_headers(input: &OneOrList<Vec<u8>>) -> ::std::result::Result<(), ValidationError> {
    if match *input {
        One(ref x) => x.is_empty(),
        List(ref x) => x.is_empty() || x.iter().any(Vec::is_empty),
    } {
        fail_valid!("empty_header", "Header patterns must not be empty sequences");
    }

    Ok(())
}

/// Validate that every filetype definition has a way to autodetect it
///
/// XXX: Allow an exception to this if "overrides" contains a glob that matches it?
fn validate_filetype_raw(input: &FiletypeRaw) -> ::std::result::Result<(), ValidationError> {
    if input.extension.is_none() && input.header.is_none() {
        fail_valid!("no_autodetect", "Neither extension nor header set for filetype");
    }
    Ok(())
}

/// Validate that no overrides are no-ops
fn validate_override_raw(input: &OverrideRaw) -> ::std::result::Result<(), ValidationError> {
    if input.path.is_empty() && input.recurse {
        fail_valid!("noop_override", "Override has no effect");
    }
    Ok(())
}

/// Helper for fields which can contain one entry or a list of entries to keep the configuration
/// file clean and easy to edit.
///
/// **TODO:** Custom ser/de impl to round-trip a bare `T` in TOML as `vec![T]` so both the file and
/// the code which consumes the config can be clean.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrList<T> {
    /// Allow `T` as shorthand for `[T]` in the TOML
    One(T),
    /// Allow more than one `T` in the TOML
    List(Vec<T>),
}
use OneOrList::{One, List};

/// Definition of `[[filetype]]` tables.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_filetype_raw"))]
pub struct FiletypeRaw {
    /// An identifier that can be referenced by `container`
    ///
    /// **TODO:** Validate this is unique
    #[validate(length(min = 1, message = "'id' must not be an empty string if present"))]
    pub id: Option<String>,
    /// The `id` of another filetype that this is a specialization of.
    /// (eg. OpenDocument and CBZ are specialized forms of Zip files.)
    ///
    /// **TODO:** Validate this references a valid `id`
    #[validate(length(min = 1, message = "'container' must not be an empty string if present"))]
    pub container: Option<String>,
    /// A human-readable description for use in status messages
    #[validate(length(min = 1, message= "'description' must not be an empty string if present"))]
    pub description: String,
    /// One or more extensions to identify the file by
    #[validate(custom = "validate_exts")]
    pub extension: Option<OneOrList<String>>,
    /// One or more headers to identify the file type by
    #[validate(custom = "validate_headers")]
    pub header: Option<OneOrList<Vec<u8>>>,
    /// The number of bytes to skip before attempting to match the header
    ///
    /// Assumed to be zero if omitted.
    ///
    /// **TODO:** Decide whether to support negative values to indicate trailers/footers.
    #[serde(default)]
    pub header_offset: usize,
    /// An identifier for a built-in handler or `[[handler.*]]` entry.
    #[validate(length(min = 1, message="'handler' must be non-empty"))]
    pub handler: String,
    /// A special case for the image verifier
    ///
    /// **TODO:** Refactor to either remove this or turn it into a HashMap for arbitrary keys
    /// passed to builtin handlers and exposed to the argv string substitution.
    #[serde(default)]
    pub multipage: bool,
}

/// Definition of `[[override]]` tables.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_override_raw"))]
pub struct OverrideRaw {
    /// A globbing pattern for files this rule should match
    #[validate(length(min = 1, message = "Globbing pattern must not be empty"))]
    pub path: String,
    /// The status message to display if this override matches a path.
    /// May be omitted to avoid displaying a message.
    #[validate(length(min = 1, message = "If provided, 'message' must not be empty"))]
    pub message: Option<String>,
    /// If specified, a `handler` to apply to the path instead of relying on autodetection.
    ///
    /// **TODO:** Decide how this should interact with directories.
    ///
    /// **TODO:** Rename the TOML field to `handler` for consistency once I no longer need to
    ///           maintain compatibility with unchanged Python code.
    #[serde(rename="type")]
    #[validate(length(min = 1, message = "An empty string is not a valid handler ID"))]
    pub handler: Option<String>,
    /// If `false` and `path` matches a directory, do not descend into it.
    #[serde(default = "bool_true_default")]
    pub recurse: bool,
}

/// Definition of `[[handler.*]]` tables.
#[derive(Debug, Deserialize, Validate)]
pub struct HandlerRaw {
    /// A template for the command to invoke via `[std::process::Command]`.
    ///
    /// The following subtitution tokens are available:
    ///
    /// * `{path}`: The path to the file to be validated.
    /// * `{devnull}`: The path to `/dev/null` or equivalent, suitable for subprocesses which
    ///    insist on producing an output file when used to check for errors.
    ///
    /// To simplify the common case, `{path}` will be appended to the end of the `Vec` if no
    /// entries contain substitution tokens.
    ///
    /// **TODO:** Validate that argv[0] does not contain a substitution token
    #[validate(length(min = 1, message = "'argv' must not be empty"))]
    pub argv: Vec<String>,
    /// If present and non-empty, the command will be considered to have failed if its output to
    /// `stderr` contains the given string, even if it returns an exit code that indicates success.
    ///
    /// Being present but empty is considered an error to avoid allowing an "unintended state that
    /// matches everything" bug slipping in.
    #[serde(default)]
    #[validate(length(min = 1, message = "If provided, 'fail_if_stderr' must not be empty"))]
    pub fail_if_stderr: Option<String>,
}

/// Root of the configuration schema
///
#[derive(Debug, Deserialize, Validate)]
pub struct RootRaw {
    /// A list of filetype definitions, including mappings to handlers.
    #[validate]
    #[serde(rename="filetype")]
    pub filetypes: Vec<FiletypeRaw>,
    /// A list of rules for overriding `filetypes` or excluding folder for specific globs.
    #[validate]
    #[serde(rename="override")]
    pub overrides: Vec<OverrideRaw>,
    /// A list of *external* handler definitions to be used by `filetypes` and `overrides`.
    /// (This list includes only subprocesses, not built-in handlers)
    ///
    /// * TODO: Validate that there exists no "empty string" key
    /// * TODO: Work around #[validate] not being implemented for hashmaps
    #[serde(rename="handler")]
    pub handlers: HashMap<String, HandlerRaw>,
}

/// Reformat the errors from the validator crate for display to the user
///
/// **TODO:** Tidy up this hacky code
pub fn format_validation_errors(errors: ValidationErrors) -> anyhow::Error {
    #![allow(clippy::let_underscore_must_use)]
    use validator::ValidationErrorsKind::{List, Field};

    let mut out_str = String::with_capacity(80);
    let _ = writeln!(&mut out_str, "Errors found in the configuration file:");
    for (section, sec_errors) in errors.into_errors() {
        #[allow(clippy::wildcard_enum_match_arm)]
        match sec_errors {
            List(map) => {
                for (idx, record_errors) in map {
                    for (field, error) in record_errors.into_errors() {
                        match error {
                            Field(x) => {
                                for err in x {
                                    let _ = writeln!(&mut out_str,
                                        "  {} entry {:>3}, {:>10} field: {}",
                                        section, idx, field,
                                        err.message.as_ref().unwrap_or(&err.code));
                                }
                            }
                            x => { let _ = writeln!(&mut out_str, "{:#?}", x); },
                        }
                    }
                }
            },
            x => { let _ = writeln!(&mut out_str, "{:#?}", x); },
        }
    }
    anyhow!(out_str)
}

/// Temporary hack for reformatting errors from handler validation
///
/// **TODO:** Replace this with a proper nested validation implementation
pub fn format_handler_validation_errors(errors: ValidationErrors) -> anyhow::Error {
    #![allow(clippy::let_underscore_must_use)]
    use validator::ValidationErrorsKind::Field;

    let mut out_str = String::with_capacity(80);
    let _ = writeln!(&mut out_str, "Errors found in the configuration file:");
    for (field, record_errors) in errors.into_errors() {
        #[allow(clippy::wildcard_enum_match_arm)]
        match record_errors {
            Field(x) => {
                for err in x {
                    let _ = writeln!(&mut out_str,
                        "  handler entry, {:>10} field: {}",
                        field, err.message.as_ref().unwrap_or(&err.code));
                }
            }
            x => { let _ = writeln!(&mut out_str, "{:#?}", x); },
        }
    }
    anyhow!(out_str)
}

/// Parse and validate the given `verifiers.toml` text
pub fn parse(toml_str: &str) -> Result<RootRaw> {
    let parsed: RootRaw = toml::from_str(toml_str)
        .with_context(|| "Error parsing configuration file")?;
    parsed.validate().map_err(format_validation_errors)?;

    // Check all [handler.*] tables for empty argv fields
    #[allow(clippy::explicit_iter_loop)]
    for (key, handler) in parsed.handlers.iter() {
        if key == "" {
            warn!("Handler ID is empty string");
        }
        handler.validate().map_err(format_handler_validation_errors)?;
    }
    // TODO: Use a Result for all other failures too, instead of `warn!`.

    // Check for typos in handler fields
    for filetype in parsed.filetypes.iter()
            .filter(|x| !parsed.handlers.contains_key(&x.handler)) {
        warn!("Unrecognized handler for {}: {}", filetype.description, filetype.handler);
    }

    // TODO: Report this allow() as a bug (The iter() is necessary to avoid a partial move error)
    #[allow(clippy::explicit_iter_loop)]
    for override_ in parsed.overrides.iter() {
        // Check for typos in handler fields
        if let Some(handler) = override_.handler.as_deref() {
            if !parsed.handlers.contains_key(handler) {
                warn!("Unrecognized handler for override {:#?}: {}",
                    override_.path, handler);
            }

        }
    }

    Ok(parsed)
}

// Tests go below the code where they'll be out of the way when not the target of attention
#[cfg(test)]
mod tests {
    use super::CliOpts;

    // TODO: Nested validation being opt-in via #[validate] is a footgun (you could have everything
    //       set up, but forget or comment out that one line and it'd still look like it should
    //       work), so write some unit tests to ensure nested validation is happening.

    // TODO: Unit test that struct-level validators are getting called properly.

    #[test]
    /// Verify that
    fn test_something() {
        // TODO: Test something
    }
}
