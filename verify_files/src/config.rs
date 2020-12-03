/// Definitions for the `verifiers.toml` configuration file.

// Standard library imports
use std::collections::HashMap;

// 3rd-party crate imports
use serde_derive::Deserialize;

/// Helper to reverse Serde's usual behaviour for a `bool` with `default`
fn bool_true_default() -> bool { true }

/// Helper for fields which can contain one entry or a list of entries to keep the configuration
/// file clean and easy to edit.
///
/// **TODO:** Custom ser/de impl to round-trip a bare `T` in TOML as `vec![T]` so both the file and
/// the code which consumes the config can be clean.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrList<T> {
    /// Allow `T` as shorthand for `[T]` in the TOML
    One(T),
    /// Allow more than one `T` in the TOML
    List(Vec<T>),
}

/// Definition of `[[filetype]]` tables.
///
/// **TODO:** Decide on the best place to raise a warning about invalid definitions.
///           (eg. Neither `extension` nor `header`, unrecognized `handler`, unrecognized
///           `container`, `header` present but empty, `id` present but not unique, etc.)
#[derive(Debug, Deserialize)]
pub struct FiletypeRaw {
    /// An identifier that can be referenced by `container`
    pub id: Option<String>,
    /// The `id` of another filetype that this is a specialization of.
    /// (eg. OpenDocument and CBZ are specialized forms of Zip files.)
    pub container: Option<String>,
    /// A human-readable description for use in status messages
    pub description: String,
    /// An extension to identify the file by
    pub extension: Option<OneOrList<String>>,
    /// A header to identify the file by
    pub header: Option<OneOrList<Vec<u8>>>,
    /// The number of bytes to skip before attempting to match the header
    pub header_offset: Option<usize>,
    /// An identifier for a built-in handler or `[[handler.*]]` entry.
    pub handler: String,
    /// A special case for the image verifier
    ///
    /// **TODO:** Refactor to either remove this or turn it into a HashMap for arbitrary keys
    /// passed to builtin handlers and exposed to the argv string substitution.
    #[serde(default)]
    pub multipage: bool,
}

/// Definition of `[[override]]` tables.
///
///
/// **TODO:** Decide on the best place to raise a warning about invalid definitions.
/// (eg. Invalid `handler`, neither `handler` nor `recurse` specified, etc.)
#[derive(Debug, Deserialize)]
pub struct OverrideRaw {
    /// A globbing pattern for files this rule should match
    pub path: String,
    /// The status message to display if this override matches a path
    pub message: Option<String>,
    /// If specified, a `handler` to apply to the path instead of relying on autodetection.
    ///
    /// **TODO:** Decide how this should interact with directories.
    ///
    /// **TODO:** Rename in the file for consistency.
    #[serde(rename="type")]
    pub handler: Option<String>,
    /// If `false` and `path` matches a directory, do not descend into it.
    #[serde(default = "bool_true_default")]
    pub recurse: bool,
}

/// Definition of `[[handler.*]]` tables.
///
///
/// **TODO:** Decide on the best place to raise a warning about invalid definitions.
/// (eg. Invalid `handler`, neither `handler` nor `recurse` specified, etc.)
#[derive(Debug, Deserialize)]
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
    pub argv: Vec<String>,
    /// If present, the command will be considered to have failed if its output to `stderr`
    /// contains the given string, even if it returns an exit code that indicates success.
    pub fail_if_stderr: Option<String>,
}

/// Root of the configuration schema
///
#[derive(Debug, Deserialize)]
pub struct RootRaw {
    /// A list of filetype definitions, including mappings to handlers.
    #[serde(rename="filetype")]
    pub filetypes: Vec<FiletypeRaw>,
    /// A list of rules for overriding `filetypes` or excluding folder for specific globs.
    #[serde(rename="override")]
    pub overrides: Vec<OverrideRaw>,
    /// A list of *external* handler definitions to be used by `filetypes` and `overrides`.
    /// (This list includes only subprocesses, not built-in handlers)
    #[serde(rename="handler")]
    pub handlers: HashMap<String, HandlerRaw>,
}
