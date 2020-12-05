//! Definitions for the `verifiers.toml` configuration file.
//!
//! Invoke this machinery via the [`parse`] function.

// Standard library imports
use std::collections::HashMap;
use std::fmt::Write;

// 3rd-party crate imports
use log::warn;
use anyhow::{anyhow, Context, Result}; // It's an internal API, so no need for thiserror yet.
use serde_derive::{Serialize, Deserialize};
use validator::{Validate, ValidationError, ValidationErrors};

// ----==== Helpers for Schema ====----

/// Wrapper to compact the repeated boilerplate of attaching messages to a custom
/// validation failure
macro_rules! fail_valid {
    ($code:expr, $msg:expr) => {
        let mut err = ValidationError::new($code);
        err.message = Some($msg.into());
        return Err(err);
    }
}

/// Helper to reverse Serde's usual behaviour for a `bool` with `default`
fn bool_true_default() -> bool { true }

/// Validator: `argv[0]` doesn't contain a substitution token (as a safety net)
fn validate_argv(argv: &[String]) -> ::std::result::Result<(), ValidationError> {
    if let Some(argv0) = argv.get(0) {
        if argv0.contains('{') {
            fail_valid!("argv0_subst", "argv[0] cannot contain substitution tokens");
        }
    }
    Ok(())
}

/// Validator: verify structural correctness of extension definitions
fn validate_exts(input: &OneOrList<String>) -> ::std::result::Result<(), ValidationError> {
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

/// Validator: no header definitions are empty strings
fn validate_headers(input: &OneOrList<Vec<u8>>) -> ::std::result::Result<(), ValidationError> {
    if match *input {
        One(ref x) => x.is_empty(),
        List(ref x) => x.is_empty() || x.iter().any(Vec::is_empty),
    } {
        fail_valid!("empty_header", "Header patterns must not be empty sequences");
    }

    Ok(())
}

/// Validator: every filetype definition includes a way to autodetect it
///
/// **XXX:** Allow an exception to this if "overrides" contains a glob that matches it?
fn validate_filetype_raw(input: &FiletypeRaw) -> ::std::result::Result<(), ValidationError> {
    if input.extension.is_none() && input.header.is_none() {
        fail_valid!("no_autodetect", "Neither extension nor header set for filetype");
    }
    Ok(())
}

/// Validator: no overrides are no-ops
fn validate_override_raw(input: &OverrideRaw) -> ::std::result::Result<(), ValidationError> {
    // Disabling recursion is a non-default effect
    if !input.recurse {
        return Ok(());
    }

    // Forcing a handler is a non-default effect
    if let Some(ref handler) = input.handler {
        if !handler.is_empty() {
            return Ok(());
        }
    }
    fail_valid!("noop_override", "Override has no effect");
}

/// Validator: filetype IDs are unique
fn validate_root_raw(input: &RootRaw) -> ::std::result::Result<(), ValidationError> {
    let mut seen = Vec::with_capacity(input.filetypes.len());
    let mut dupes = Vec::with_capacity(4);

    for record in &input.filetypes {
        if let Some(ref id) = record.id {
            if seen.contains(&id) {
                dupes.push(id);
            }
            seen.push(id);
        }
    }
    if !dupes.is_empty() {
        fail_valid!("dupe_filetype_id", format!("Duplicate filetype IDs: {:?}", dupes));
    }
    Ok(())
}

/// Helper to add support for using `#[validate]` nesting to `HashMap`
///
/// (Works by exploiting how validator is implemented using macros and, as such, can duck-type its
/// method resolution.)
///
/// Thanks to [@Kaiser1989](https://github.com/Keats/validator/issues/83#issuecomment-732006938)
/// for this trick.
pub trait ValidateExtensions {
    /// See [`Validate::validate`]
    fn validate(&self) -> Result<(), ValidationErrors>;
}
impl<K, V: Validate> ValidateExtensions for HashMap<K, V> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        for value in self.values() { value.validate()? }
        Ok(())
    }
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

// ----==== Configuration Schema ====----

/// Definition of `[[filetype]]` tables.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_filetype_raw"))]
pub struct FiletypeRaw {
    /// An identifier that can be referenced by `container`
    #[validate(length(min = 1, message = "'id' must not be an empty string if present"))]
    pub id: Option<String>,
    /// The `id` of another filetype that this is a specialization of.
    /// (eg. OpenDocument and CBZ are specialized forms of Zip files.)
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
    /// An identifier for a built-in handler or `[handler.*]` entry.
    ///
    /// **TODO:** Turn this into a `OneOrList<String>` to allow fallback chains for .exe/.bin/etc.
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

/// Definition of `[handler.*]` tables.
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
    #[validate(length(min = 1, message = "'argv' must not be empty"), custom = "validate_argv")]
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
#[validate(schema(function = "validate_root_raw"))]
pub struct RootRaw {
    /// A list of filetype definitions, including mappings to handlers.
    #[validate]
    #[serde(rename="filetype", default)]
    pub filetypes: Vec<FiletypeRaw>,

    /// A list of rules for overriding `filetypes` or excluding folder for specific globs.
    #[validate]
    #[serde(rename="override", default)]
    pub overrides: Vec<OverrideRaw>,

    /// A list of *external* handler definitions to be used by `filetypes` and `overrides`.
    /// (This list includes only subprocesses, not built-in handlers)
    #[validate]
    #[serde(rename="handler", default)]
    pub handlers: HashMap<String, HandlerRaw>,
}

// ----==== Parsing Functions ====----

/// Reformat [`ValidationErrors`] for display to the user
///
/// (Because the default [`Display`](std::fmt::Display) implementation just aliases it
/// to [`Debug`])
///
/// **TODO:** Tidy up this hacky code
///
/// **TODO:** Special-case `__all__` entry name to make certain errors clearer
pub fn format_validation_errors(errors: ValidationErrors) -> anyhow::Error {
    #![allow(clippy::let_underscore_must_use)]
    use validator::ValidationErrorsKind::{Field, List, Struct};

    let mut out_str = String::with_capacity(80);
    let _ = writeln!(&mut out_str, "Errors found in the configuration file:");
    for (section, sec_errors) in errors.into_errors() {
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
                            x @ Struct(..) | x @ List(..) => {
                                let _ = writeln!(&mut out_str, "{:#?}", x);
                            },
                        }
                    }
                }
            },
            Struct(record_errors) => {
                for (field, error) in record_errors.into_errors() {
                    match error {
                        Field(x) => {
                            for err in x {
                                let _ = writeln!(&mut out_str,
                                    "  {} entry, {:>10} field: {}",
                                    section, field,
                                    err.message.as_ref().unwrap_or(&err.code));
                            }
                        }
                        x @ Struct(..) | x @ List(..) => {
                            let _ = writeln!(&mut out_str, "{:#?}", x);
                        },
                    }
                }
            },
            Field(field_error) => {
                for err in field_error {
                    let _ = writeln!(&mut out_str, "  {} entry: {}",
                        section, err.message.as_ref().unwrap_or(&err.code));
                }
            },
        }
    }
    anyhow!(out_str)
}

/// Parse and validate the given `verifiers.toml` text
pub fn parse(toml_str: &str) -> Result<RootRaw> {
    // Parse and perform all validation where the outcome couldn't change as a result of a fallback
    // chain injecting new values.
    let parsed: RootRaw = toml::from_str(toml_str)
        .with_context(|| "Error parsing configuration file")?;
    parsed.validate().map_err(format_validation_errors)?;
    // TODO: Use a Result for all other failures too, instead of `warn!`.

    // Check for `container` values that don't match any filetype IDs
    // TODO: Refactor this when I'm not about to fall asleep
    let seen_ids: Vec<&str> = parsed.filetypes.iter().filter_map(|x| x.id.as_deref()).collect();
    // TODO: Report this allow() as a bug (The iter() is necessary to avoid a partial move error)
    #[allow(clippy::explicit_iter_loop)]
    for filetype in parsed.filetypes.iter() {
        if let Some(ref container) = filetype.container {
            if !seen_ids.contains(&container.as_str()) {
                warn!("Invalid container ID for {}: {}", filetype.description, container);
            }
        }
    }

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

// ----==== Tests ====----

#[cfg(test)]
mod tests {
    use super::*;

    /// Quick helper to deduplicate basic assertions
    ///
    /// **TODO:** Assert more
    fn assert_validation_result(toml_str: &str, section: &str) {
        let result = do_validate(toml_str).expect_err("Validation should error out").into_errors();
        result.get(section).expect(&format!("Get section {} from error response", section));
    }

    fn do_validate(toml_str: &str) -> std::result::Result<(), ValidationErrors> {
        let parsed: RootRaw = toml::from_str(toml_str).unwrap();
        parsed.validate()
    }

    // TODO: Unit test for the checks that currently `warn!`

    /// Ensure that duplicate filetype IDs get caught
    #[test]
    fn test_rootraw_duplicate_id() {
        assert_validation_result(r#"
            [[filetype]]
            id = "foo"
            description = "Test description 1"
            handler = "foo"
            extension = ".foo"

            [[filetype]]
            id = "foo"
            description = "Test description 2"
            handler = "bar"
            extension = ".bar"
        "#, "__all__");
    }

    /// Verify that nested validation is occurring
    ///
    /// (Because it's so easy to accidentally drop a `#[validate]` and not notice)
    #[test]
    fn test_nested_validation() {
        // Filetype with an invalid description to trigger nested validation failure
        assert_validation_result(r#"
                [[filetype]]
                description = ""
                handler = "foo"
                extension = ".foo"
            "#, "filetype");

        // Override with an invalid description to trigger nested validation failure
        assert_validation_result(r#"
                [[override]]
                path = ""
            "#, "override");

        // Handler with an invalid argv to trigger nested validation failure
        assert_validation_result(r#"
                [handler.foo]
                argv = []
            "#, "handler");
    }

    /// Verify that the struct-level validators are running correctly
    #[test]
    fn test_struct_validation() {
        // Filetype with no way to autodetect it
        assert_validation_result(r#"
                [[filetype]]
                description = "Test description"
                handler = "foo"
            "#, "filetype");

        // Override that does nothing
        assert_validation_result(r#"
                [[override]]
                path = "quux"
            "#, "override");
    }

    /// Verify that all sections are optional
    /// (It's not the low-level parser's job to know whether there's a fallback chain)
    #[test]
    fn test_optional_sections() {
        // The parser shouldn't assume there's no fallback chain
        do_validate(r#""#).expect("Empty config files should parse just fine");

        do_validate(r#"
                [[filetype]]
                description = "Test description"
                handler = "foo"
                extension = ".foo"
            "#).expect("The parser should accept a filetype that may rely only on builtins");

        do_validate(r#"
                [[override]]
                path = "bar"
                recurse = false
            "#).expect("The parser should assume a lone override relies on a fallback chain");

        do_validate(r#"
                [handler.baz]
                argv = [ "baz" ]
            "#).expect("The parser should assume a lone handler is referenced outside its sight");
    }

    /// Ensure the continued presence of a behaviour I'm not sure how I achieved
    #[test]
    fn test_rejects_empty_handler_id() {
        let parsed: Result<RootRaw, _> = toml::from_str(r#"[handler.""]"#);
        parsed.expect_err("Empty table names should be rejected");
    }

    /// Ensure the `argv[0]` subsitution rejection doesn't break
    #[test]
    fn test_rejects_substition_in_argv0() {
        do_validate(r#"
                [handler.foobar]
                argv = [ "{foobar}" ]
            "#).expect_err("argv[0] should reject substitution tokens (1)");
        do_validate(r#"
                [handler.foobar]
                argv = [ "foo{bar}" ]
            "#).expect_err("argv[0] should reject substitution tokens (2)");
        do_validate(r#"
                [handler.foobar]
                argv = [ "foo{bar}baz" ]
            "#).expect_err("argv[0] should reject substitution tokens (3)");
    }

    /// Ensure that `recurse=true` being default for `[[override]]` doesn't accidentally changed
    #[test]
    fn test_overrideraw_recurse_default() {
        let parsed: RootRaw = toml::from_str(r#"
            [[override]]
            path = "foo"
            type = "bar"
        "#).unwrap();
        let entry = parsed.overrides.iter().next().unwrap();

        // Control checks
        assert_eq!(&entry.path, "foo");
        assert_eq!(entry.handler.as_deref(), Some("bar"));
        // Actual test
        assert_eq!(entry.recurse, true);
    }
}
