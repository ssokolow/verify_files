//! Definitions for the `verifiers.toml` configuration file.
//!
//! Invoke this machinery via the [`parse`] function.
//!
//! **NOTE:** Uses `BTreeMap` instead of `HashMap` to ensure the data will serialize to TOML in
//! sorted order.
//!
//! **TODO:** Consider using the [`types`
//! module](https://docs.rs/ignore/0.4.17/ignore/types/index.html) from the `ignore` crate and
//! enabling its default type definitions so error messages can give a slightly more human-friendly
//! name for various "no handler registered for this type" situations.

// Standard library imports
use std::collections::BTreeMap;
use std::fmt::Write;
use std::ops::Not;
use std::result::Result as StdResult;

// 3rd-party crate imports
use anyhow::{anyhow, Context, Result}; // It's an internal API, so no need for thiserror yet.
use log::warn;
use serde_derive::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

// ----==== Helpers for Schema ====----

/// Wrapper to compact the repeated boilerplate of attaching messages to a custom
/// validation failure
macro_rules! fail_valid {
    ($code:expr, $msg:expr) => {{
        let mut err = ValidationError::new($code);
        err.message = Some($msg.into());
        return Err(err);
    }};
}

/// Helper for Serde's `skip_serializing_if`
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(int: &usize) -> bool {
    *int == 0
}

/// Validator: `argv[0]` doesn't contain any substitution tokens (as a safety net)
fn validate_argv(argv: &[String]) -> StdResult<(), ValidationError> {
    if let Some(argv0) = argv.get(0) {
        if argv0.contains('{') {
            fail_valid!(
                "argv0_subst",
                format!("argv[0] cannot contain substitution tokens: {}", argv0)
            );
        }
    }
    Ok(())
}

/// Validator: verify structural correctness of `extension` fields
fn validate_exts(input: &OneOrList<String>) -> StdResult<(), ValidationError> {
    if input.is_empty() || input.iter().any(String::is_empty) {
        fail_valid!("empty_ext", "Extensions may not be empty strings");
    }

    let exts: Vec<_> = input.iter().map(String::as_str).filter(|x| x.starts_with('.')).collect();
    if !exts.is_empty() {
        fail_valid!(
            "no_period_ext",
            format!("Extensions must not start with a period: {}", exts.join(", "))
        );
    }

    Ok(())
}

/// Validator: none of the `handler` fields contain empty strings
fn validate_handlers(input: &OneOrList<String>) -> StdResult<(), ValidationError> {
    if input.is_empty() || input.iter().any(String::is_empty) {
        fail_valid!("empty_handler", "Handler names must not be empty sequences");
    }

    Ok(())
}

/// Validator: none of the `header` fields contain empty strings
fn validate_headers(input: &OneOrList<Vec<u8>>) -> StdResult<(), ValidationError> {
    if input.is_empty() || input.iter().any(Vec::is_empty) {
        fail_valid!("empty_header", "Header patterns must not be empty sequences");
    }

    Ok(())
}

/// Validator: every filetype definition maps an autodetection method to a handler
///
/// **XXX:** Have overrides map to filetypes instead of handlers and allow an exception to this if
/// "overrides" contains a glob that matches it?
fn validate_filetype(input: &Filetype) -> StdResult<(), ValidationError> {
    if input.extension.is_none() && input.header.is_none() {
        fail_valid!(
            "no_autodetect",
            format!("Neither extension nor header set for filetype: {}", input.description)
        );
    }
    if input.handler.is_none() && input.container.is_none() {
        fail_valid!(
            "no_handler",
            format!("Neither handler nor container set for filetype: {}", input.description)
        );
    }
    Ok(())
}

/// Validator: none of the overrides are no-ops
fn validate_override(input: &Override) -> StdResult<(), ValidationError> {
    // Ignoring is a non-default effect
    if input.ignore {
        return Ok(());
    }

    // Forcing a handler is a non-default effect
    if let Some(ref handler) = input.handler {
        if !handler.is_empty() {
            return Ok(());
        }
    }
    fail_valid!("noop_override", format!("Override has no effect: {}", input.path));
}

/// Validator: all filetypes have sane `container` dependencies
fn validate_root(input: &Root) -> StdResult<(), ValidationError> {
    for (id, mut filetype) in &input.filetypes {
        // Don't bother allocating the dep_chain vec for entries without `container`
        if filetype.container.is_none() {
            continue;
        }

        // Pre-allocate for the typical case of only one iteration plus the push() for error join()
        // (Something like "cbz -> zip -> cbz")
        let mut dep_chain = Vec::with_capacity(3);
        dep_chain.push(id.as_str());

        while let Some(container) = filetype.container.as_deref() {
            let cycle = dep_chain.contains(&container);
            dep_chain.push(&container);
            if cycle {
                fail_valid!(
                    "container_cycle",
                    format!("Cyclical 'container' dependency: {}", dep_chain.join(" -> "))
                );
            }
            if let Some(container_filetype) = input.filetypes.get(container) {
                filetype = container_filetype
            } else {
                fail_valid!(
                    "container_not_found",
                    format!("'container' for {} not found: {}", id, container)
                );
            }
        }
    }
    Ok(())
}

/// Validator: If present, the `sources` field must contain valid URLs
///
/// **TODO:** Look into how much weight it would add to validate the format of these further.
fn validate_sources(input: &OneOrList<String>) -> StdResult<(), ValidationError> {
    if input.is_empty() || input.iter().any(String::is_empty) {
        fail_valid!("empty_handler", "Source list must be absent or contain non-empty strings");
    }

    // Ensure users who already need help don't have to deal with esoteric protocols
    for url in input.iter() {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            fail_valid!("invalid_url", "Only HTTP and HTTPS URLs are supported as sources");
        }
    }
    Ok(())
}

/// Helper to add support for using `#[validate]` nesting to `BTreeMap`
///
/// (As I understand it, this works by exploiting how validator is implemented using macros and,
/// as such, can duck-type its method resolution.)
///
/// Thanks to [@Kaiser1989](https://github.com/Keats/validator/issues/83#issuecomment-732006938)
/// for this trick.
pub trait ValidateExtensions {
    /// See [`Validate::validate`]
    fn validate(&self) -> Result<(), ValidationErrors>;
}
impl<K, V: Validate> ValidateExtensions for BTreeMap<K, V> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        for value in self.values() {
            value.validate()?
        }
        Ok(())
    }
}

/// Helper for fields which can contain one entry or a list of entries to keep the configuration
/// file clean and easy to edit.
///
/// **TODO:** Custom ser/de impl to round-trip a bare `T` in TOML as `vec![T]` so both the file and
/// the code which consumes the config can be clean.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneOrList<T> {
    /// Allow `T` as shorthand for `[T]` in the TOML
    One(T),
    /// Allow more than one `T` in the TOML
    List(Vec<T>),
}
use OneOrList::{List, One};

impl<T> ::std::ops::Deref for OneOrList<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        match *self {
            One(ref x) => std::slice::from_ref(x),
            List(ref x) => x.as_slice(),
        }
    }
}

// ----==== Configuration Schema ====----

/// Definition of `[[filetype]]` tables.
#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_filetype"))]
pub struct Filetype {
    /// The id of another filetype that this is a specialization of.
    /// (eg. OpenDocument and CBZ are specialized forms of Zip files.)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, message = "'container' must not be an empty string if present"))]
    pub container: Option<String>,

    /// A human-readable description for use in status messages
    #[validate(length(min = 1, message = "'description' must not be an empty string"))]
    pub description: String,

    /// One or more extensions to identify the file by
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_exts")]
    pub extension: Option<OneOrList<String>>,

    /// An identifier for a built-in handler or `[handler.*]` entry.
    ///
    /// If specified as a list, it indicates a fallback chain from more desirable/thorough
    /// validators to less desirable/thorough validators **for the same file type**.
    ///
    /// The first validator that is available but reports failure will stop the fallback and the
    /// file will be considered as corrupted.
    ///
    /// If fallback is necessary to tell apart several formats which share the same extension
    /// and/or header (eg. `.exe` possibly being multiple different kinds of self-extracting
    /// archives), then specify multiple `[filetype.*]` sections with the same or overlapping
    /// `extension` and `header` content.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_handlers")]
    pub handler: Option<OneOrList<String>>,

    /// One or more headers to identify the file type by
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_headers")]
    pub header: Option<OneOrList<Vec<u8>>>,

    /// The number of bytes to skip before attempting to match the header
    ///
    /// Assumed to be zero if omitted.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub header_offset: usize,

    /// A special case for the image verifier
    ///
    /// **TODO:** Refactor to either remove this or turn it into a BTreeMap for arbitrary keys
    /// passed to builtin handlers and exposed to the argv string substitution.
    #[serde(default, skip_serializing_if = "Not::not")]
    pub multipage: bool,
}

/// Definition of `[[override]]` tables.
#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_override"))]
pub struct Override {
    /// A globbing pattern for files this rule should match
    #[validate(length(min = 1, message = "Globbing pattern must not be empty"))]
    pub path: String,

    /// If specified, a file `handler` to apply to the path instead of relying on autodetection.
    ///
    /// Has no effect when the glob matches a directory.
    ///
    /// **NOTE:** At some point, I may need to extend the design to also support handlers that
    /// take a *directory* path as input without risking feeding directories with file-like names
    /// to handlers that only expect files.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_handlers")]
    pub handler: Option<OneOrList<String>>,

    /// If `true`, don't process files or descend into directories matching the given glob.
    ///
    /// **TODO:** Disentangle `handler` and `ignore` overrides. Aside from "make invalid states
    /// unrepresentable" (custom handler and ignore=true), using the `ignore` crate means that the
    /// `message` field can't apply to overrides, so it makes more sense to do something like
    /// having an ignores `Vec` and a handler overrides `BTreeMap` at the top level.
    #[serde(default, skip_serializing_if = "Not::not")]
    pub ignore: bool,

    /// The status message to display if this override matches a path.
    /// May be omitted to avoid displaying a message.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, message = "If provided, 'message' must not be empty"))]
    pub message: Option<String>,
}

/// Definition of `[handler.*]` tables.
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Handler {
    /// A template for the command to invoke via `[std::process::Command]`.
    ///
    /// The following substitution tokens are available:
    ///
    /// * `{path}`: The path to the file to be validated.
    /// * `{devnull}`: The path to `/dev/null` or equivalent, suitable for subprocesses which
    ///    insist on producing an output file when used to check for errors.
    ///
    /// To simplify the common case, `{path}` will be appended to the end of the `Vec` if no
    /// entries contain substitution tokens.
    #[validate(length(min = 1, message = "'argv' must not be empty"), custom = "validate_argv")]
    pub argv: Vec<String>,

    /// A human-readable description for use in status messages instead of the command name from
    /// `argv[0]` when indicating what needs to be installed.
    ///
    /// Should include the definite article "the" if necessary to fit in a message like
    /// "Could not find {description}. Please install it from one of the following URLs:" but,
    /// by convention, it should also be capitalized suitably for display alone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, message = "If provided, 'description' must not be empty"))]
    pub description: Option<String>,

    /// If present and non-empty, the command will be considered to have failed if its output to
    ///
    /// **NOTE:** At some point, I may need to extend the design to also support handlers that
    /// take a *directory* path as input without risking feeding directories with file-like names
    /// to handlers that only expect files.
    /// `stderr` contains the given string, even if it returns an exit code that indicates success.
    ///
    /// Being present but empty is considered an error to avoid allowing an "unintended state that
    /// matches everything" bug slipping in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, message = "If provided, 'fail_if_stderr' must not be empty"))]
    pub fail_if_stderr: Option<String>,

    /// If specified, one or more URLs from which the handler can be installed.
    ///
    /// By convention:
    ///
    /// * The first item in a multi-element list should point to the project's website
    /// * Links should point to the stable, version-agnostic URL closest to the actual download
    ///   without immediately initiating a download.
    /// * If the project's website includes links to builds for multiple platforms, link to the
    ///   page containing them instead.
    ///
    /// This means that, in a case like 7-Zip and p7zip where the official project is Windows-only,
    /// the link for Windows users should come first while, for projects like RPM which are ported
    /// from Linux to Windows, the link for Linux users should come first.
    ///
    /// It is acceptable to link to the website for Cygwin if the only suitable Windows port is
    /// provided as part of Cygwin.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom = "validate_sources")]
    pub sources: Option<OneOrList<String>>,
}

/// Root of the configuration schema
///
#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_root"))]
pub struct Root {
    /// A list of filetype definitions, including mappings to handlers.
    ///
    /// It is represented as a hashmap to ensure that each filetype has a unique identifer. This
    /// is for two reasons:
    ///
    /// 1. It allows the TOML parser to handle preventing duplicate IDs.
    /// 2. It ensures that later validation stages which may not have easy access to
    ///    line numbers will have something to use in error messages.
    ///
    /// By convention, the identifier is the most natural single-word name for the
    /// file format, in lowercase alphanumeric form (eg. jpeg, mp3, 7zip, etc.) but,
    /// if more than one format has the same name, do not hesitate to use underscores
    /// to avoid ambiguity. (eg. ms_cab)
    #[validate]
    #[serde(rename = "filetype", default, serialize_with = "toml::ser::tables_last")]
    pub filetypes: BTreeMap<String, Filetype>,

    /// A list of rules for overriding `filetypes` or excluding folder for specific globs.
    #[validate]
    #[serde(rename = "override", default)]
    pub overrides: Vec<Override>,

    /// A list of *external* handler definitions to be used by `filetypes` and `overrides`.
    /// (This list includes only subprocesses, not built-in handlers)
    #[validate]
    #[serde(rename = "handler", default, serialize_with = "toml::ser::tables_last")]
    pub handlers: BTreeMap<String, Handler>,
}

// ----==== Parsing Functions ====----

/// Reformat [`ValidationErrors`] for display to the user
///
/// (Because the default [`Display`](std::fmt::Display) implementation just aliases it
/// to [`Debug`])
///
/// **TODO:** Tidy up this hacky code
///
/// **TODO:** Special-case `__all__` field name to make certain errors clearer
#[rustfmt::skip]
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
                            },
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
                        },
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
///
/// TODO: Better design for integrating the builtin handler check.
pub fn parse(toml_str: &str, is_builtin_handler: &dyn Fn(&str) -> bool) -> Result<Root> {
    // Parse and perform all validation where the outcome couldn't change as a result of a fallback
    // chain injecting new values.
    let parsed: Root =
        toml::from_str(toml_str).with_context(|| "Error parsing configuration file")?;
    parsed.validate().map_err(format_validation_errors)?;
    // TODO: Use a Result for all other failures too, instead of `warn!`.

    // Check for `container` values that don't match any filetype IDs
    for (id, filetype) in &parsed.filetypes {
        if let Some(ref container) = filetype.container {
            if !parsed.filetypes.contains_key(container.as_str()) {
                warn!("Invalid container ID for filetype {}: {}", id, container);
            }
        }
    }

    // Check for typos in filetype handler fields
    for (id, filetype) in &parsed.filetypes {
        if let Some(ref handler) = filetype.handler {
            for handler in handler
                .iter()
                .filter(|y| !(parsed.handlers.contains_key(*y) || is_builtin_handler(y.as_str())))
            {
                warn!("Unrecognized handler for filetype {}: {}", id, handler);
            }
        }
    }

    // Check for typos in override handler fields
    for override_ in &parsed.overrides {
        // Check for typos in handler fields
        if let Some(handler) = override_.handler.as_deref() {
            for handler in handler.iter().filter(|y| !parsed.handlers.contains_key(*y)) {
                warn!("Unrecognized handler for override {:#?}: {}", override_.path, handler);
            }
        }

        match override_.path.as_str() {
            "*" | "*.*" => warn!("Override with too-broad `path` glob: {}", override_.path),
            _ => {},
        }
    }

    // TODO: At a higher level (not config file parsing), decide how to implement checking for
    //       nonexistent argv0 in handlers without annoying people who don't need support for all
    //       formats installed. (Maybe a config key that you set after installation to silence
    //       pre-flight checks for formats you only want to be warned about when it encounters one
    //       it can't check? ...or maybe a command-line argument which causes it to output a report
    //       on what formats are supported but not possible and what to install to enable them.)

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
        let parsed: Root = toml::from_str(toml_str).unwrap();
        parsed.validate()
    }

    // TODO: Unit tests for the checks that currently `warn!`

    /// Verify that nested validation is occurring
    ///
    /// (Because it's so easy to accidentally drop a `#[validate]` and not notice)
    #[test]
    #[rustfmt::skip]
    fn test_nested_validation() {
        // Filetype with an invalid description to trigger nested validation failure
        assert_validation_result(r#"
                [filetype.foo]
                description = ""
                handler = "foo"
                extension = "foo"
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
    #[rustfmt::skip]
    fn test_struct_validation() {
        // Filetype with no way to autodetect it
        assert_validation_result(r#"
                [filetype.foo]
                description = "Test description"
                handler = "foo"
            "#, "filetype");

        // Override that does nothing
        assert_validation_result(r#"
                [[override]]
                path = "quux"
            "#, "override");
    }

    /// Verify that sources are checked to be superficially valid URLs
    #[test]
    #[rustfmt::skip]
    fn test_sources_validation() {
        do_validate(r#"
                [handler.foo]
                argv = ["foo"]
            "#).expect("Parsed handler definition with no sources");
        do_validate(r#"
                [handler.foo]
                argv = ["foo"]
                sources = ["http://www.example.com/"]
            "#).expect("Parsed handler definition with sources");
        do_validate(r#"
                [handler.foo]
                argv = ["foo"]
                sources = ["http://www.example.com/", "https://example.com/"]
            "#).expect("Parsed handler definition with sources");
        assert_validation_result(r#"
                [handler.foo]
                argv = ["foo"]
                sources = ""
            "#, "handler");
        assert_validation_result(r#"
                [handler.foo]
                argv = ["foo"]
                sources = []
            "#, "handler");
        assert_validation_result(r#"
                [handler.foo]
                argv = ["foo"]
                sources = [""]
            "#, "handler");
        assert_validation_result(r#"
                [handler.foo]
                argv = ["foo"]
                sources = ["foo"]
            "#, "handler");
        assert_validation_result(r#"
                [handler.foo]
                argv = ["foo"]
                sources = ["http://www.example.com/", ""]
            "#, "handler");
        assert_validation_result(r#"
                [handler.foo]
                argv = ["foo"]
                sources = ["http://www.example.com/", "ftp://example.com/"]
            "#, "handler");
    }

    /// Verify that all sections are optional
    /// (It's not the low-level parser's job to know whether there's a fallback chain)
    #[test]
    #[rustfmt::skip]
    fn test_optional_sections() {
        // The parser shouldn't assume there's no fallback chain
        do_validate(r#""#).expect("Empty config files should parse just fine");

        do_validate(r#"
                [filetype.foo]
                description = "Test description"
                handler = "foo"
                extension = "foo"
            "#).expect("The parser should accept a filetype that may rely only on builtins");

        do_validate(r#"
                [[override]]
                path = "bar"
                ignore = true
            "#).expect("The parser should assume a lone override relies on a fallback chain");

        do_validate(r#"
                [handler.baz]
                argv = [ "baz" ]
            "#).expect("The parser should assume a lone handler is referenced outside its sight");
    }

    /// Ensure the continued presence of a behaviour I'm not sure how I achieved
    #[test]
    fn test_rejects_empty_filetype_id() {
        let parsed: Result<Root, _> = toml::from_str(r#"[filetype.""]"#);
        parsed.expect_err("Empty table names should be rejected");
    }

    /// Ensure the continued presence of a behaviour I'm not sure how I achieved
    #[test]
    fn test_rejects_empty_handler_id() {
        let parsed: Result<Root, _> = toml::from_str(r#"[handler.""]"#);
        parsed.expect_err("Empty table names should be rejected");
    }

    /// Ensure the `argv[0]` subsitution rejection doesn't break
    #[test]
    #[rustfmt::skip]
    fn test_rejects_substition_in_argv0() {
        assert_validation_result(r#"
                [handler.foobar]
                argv = [ "{foobar}" ]
            "#, "handler");
        assert_validation_result(r#"
                [handler.foobar]
                argv = [ "foo{bar}" ]
            "#, "handler");
        assert_validation_result(r#"
                [handler.foobar]
                argv = [ "foo{bar}baz" ]
            "#, "handler");
    }

    /// Make sure the validation catches 'container' cycles
    #[test]
    fn test_rejects_container_cycle() {
        assert_validation_result(
            r#"
            [filetype.foo]
            description = "Foo"
            extension = "foo"
            container = "bar"

            [filetype.bar]
            description = "Bar"
            extension = "bar"
            container = "baz"

            [filetype.baz]
            description = "Baz"
            extension = "baz"
            container = "foo"
        "#,
            "__all__",
        );
    }

    /// Make sure the validation catches unknown 'container' values
    #[test]
    fn test_rejects_unknown_container() {
        assert_validation_result(
            r#"
            [filetype.foo]
            description = "Foo"
            extension = "foo"
            container = "bar"
        "#,
            "__all__",
        );
    }
}
