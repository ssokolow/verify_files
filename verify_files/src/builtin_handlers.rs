//! Built-in handlers for common file and container formats
//!
//! **NOTE:** These generally stop checking and report failure on encountering their first error,
//! because the point of the tool as a whole is to identify problem files so more specialized
//! tools can then be brought in to investigate in greater detail.
//!
//! **TODO:** Trigger and fine-tune the human-visible results of all these error cases.
//!
//! **TODO:** When I have time to figure out how best to make it play nice with the config loader's
//! sanity checks, make these optional features.
//!

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::Path;

use flate2::bufread::MultiGzDecoder;

use image::error::ImageError;
use image::ImageReader;

use lazy_static::lazy_static;

use zip::read::ZipArchive;
use zip::result::{ZipError, ZipResult};

/// The function signature for file-type handler implementations
pub type HandlerFn = fn(&Path) -> Result<(), FailureType>;

// Chosen because it's already a transitive dependency, unlike `phf`
lazy_static! {
    /// A registry of all built-in handlers, with descriptions suitable for display to end-users,
    /// keyed by the IDs exposed to the config file.
    ///
    /// (Uses a BTreeMap to control the ordering of user-visible readouts without an extra sort)
    pub static ref ALL: BTreeMap<&'static str, (&'static str, HandlerFn)> = {
        let mut m = BTreeMap::new();
        m.insert("gzip", ("GZip CRC check (built-in)", gzip as HandlerFn));
        m.insert("image", ("BMP/GIF/ICO/JPEG/PNG/PNM/TGA/TIFF handler (built-in)",
                image as HandlerFn));
        m.insert("json", ("JSON well-formedness check (built-in)", json as HandlerFn));
        m.insert("toml", ("TOML well-formedness check (built-in)", toml as HandlerFn));
        m.insert("zip", ("STORE/DEFLATE-compressed Zip CRC check (built-in)", zip as HandlerFn));
        m
    };
}

/// A return value to indicate whether a handler couldn't verify the given file because it was
/// corrupted or because it uses features not supported by the validator.
///
/// (And, as such, whether the testing system should abort or continue down the fallback chain.)
pub enum FailureType {
    /// The handler detected some form of corruption or fatal spec-noncompliance
    ///
    /// (Report the file as invalid to the user and abort the fallback chain *for this format*)
    ///
    /// This is the correct return type for fallback behaviour where multiple independent formats
    /// share the same detection characteristics, such as self-extracting archives with `.exe`
    /// stubs, as it will abort fallback within a single format such as from internal Zip handling
    /// to p7zip's more versatile Zip support.
    ///
    /// (It is also the reason that all handlers that invoke subprocesses must currently be
    /// ordered from broadest to narrowest format support, all else being equal. Without a way for
    /// a subprocess to distinguish between a bad file and an unsupported feature, all failures
    /// must be assumed to be corruption.)
    InvalidContent(/** Stringified form of the internal error message */ String),

    /// The handler does not support the provided data
    ///
    /// (Log a status message and try the next handler in the fallback chain)
    ///
    /// This is the correct return type for fallback behaviour where a single format has multiple
    /// choices for what handler should be used to validate it, as it will try the next handler in
    /// the fallback chain on failure.
    ///
    /// It is intended to allow built-in handlers with limited format support to have a higher
    /// priority in fallback chains than external handlers without treating unsupported format
    /// variations (like PPMd-compressed Zip files) as corruption.
    ///
    /// **TODO:** Decide how to handle cases where something imposes further validity constraints
    /// on its container, like EPUB, JAR, or OpenDocument only supporting STORE or DEFLATE.
    UnsupportedFormat(/** Stringified form of the internal error message */ String),

    /// The file cannot be read for some reason
    ///
    /// (Log a status message and move on to the next file)
    ///
    /// This variant is named after [`std::io::Error`] from the Rust standard library, which is
    /// almost certainly going to be the cause of it through situations such as "Access Denied",
    /// "file was deleted before we tried to read it", or "the media has bad sectors".
    IoError(/** Stringified form of the internal error message */ String),

    /// The handler reported an unexpected but recoverable error
    ///
    /// (This is for cases like [`ImageError::Limits`] and is intended as a cleaner alternative to
    /// intentionally using panicking and `catch_unwind` like try/catch to skip to the next item to
    /// be processed.)
    ///
    /// **TODO:** Be more clear about what purpose this serves, when to use it, and what result it
    /// will have.
    InternalError(/** Stringified form of the internal error message */ String),
}

/// A return value to indicate how reliable a validator's verdict of "no problems" is.
///
/// **TODO:** Decide on whether a meaningful total ordering can be had if I split
/// `DataHashAndMetaParity` so it's possible to specify data and metadata protection level
/// completely independently.
///
/// **TODO:** Decide whether this should instead serve as a metadata key that's applied to each
/// validator definition for **pre**-selection of the most reliable validator available.
pub enum Confidence {
    /// The validator checks the basic well-formedness of the data but does no further checking.
    ///
    /// (eg. Plaintext that parses as valid UTF-8, JSON or XML that parses successfully, binary
    /// formats detected to have been truncated by having internal "data length" values larger than
    /// the size of the file, formats like `tar` which checksum the metadata headers but not the
    /// data itself, etc.)
    WellFormed,
    /// The file format has only incredibly weak protections, such as odd/even parity bits, or the
    /// validator only knows how to use such checks.
    DataParity,
    /// The data chunks within the file are covered by some form of hash or checksum (eg. the CRC32
    /// checksums in a Zip file, or the MD5 hash in a FLAC file) and the validator verified it.
    ///
    /// **TODO:** Decide how to distinguish "only checks FLAC CRCs" from "checks FLAC MD5sum"
    DataHash,
    /// In addition to checking the checksum/hash, the validator exploits redundancy or parity
    /// information in the metadata to perform basic corruption checks.
    ///
    /// (eg. checking a Zip file for consistency between the fields which are present in both the
    /// local file headers and the central directory records.)
    DataHashAndMetaParity,
    /// The file has some internal hash/checksum over its entire contents (eg. an ISO image
    /// augmented by dvdisaster ECC) that the validator verified.
    FullHash,
}

/// Helper for APIs that validate lazily and need to have their `Read`-ers read through to the end
fn exhaust_reader(mut reader: impl Read) -> Result<(), io::Error> {
    let mut scratch_buffer = [0; 0xFFFF];
    loop {
        if let Err(e) = reader.read_exact(&mut scratch_buffer) {
            #[allow(clippy::wildcard_enum_match_arm)]
            return match e.kind() {
                io::ErrorKind::UnexpectedEof => Ok(()),
                _ => Err(e),
            };
        }
    }
}

/// Handler: Use the `flate2` crate to validate a stream of one or more gzipped files
///
/// **TODO:** Decide on the best API for selecting whether this should operate recursively to
/// validate the data that it must extract anyway to check the CRC.
///
/// (As a means to detect corruption that occurred before the compression was applied.)
pub fn gzip(path: &Path) -> Result<(), FailureType> {
    let reader = File::open(path).map_err(|err| FailureType::IoError(err.to_string()))?;
    exhaust_reader(MultiGzDecoder::new(BufReader::new(reader)))
        .map_err(|err| FailureType::InvalidContent(err.to_string()))
}

/// Handler: Use the `image` crate to validate the formats it supports
///
/// **TODO:** Test how thoroughly each format can be checked, and also check whether enabling WebP
/// support will validate well enough to be useful even though it doesn't support chroma yet.
pub fn image(path: &Path) -> Result<(), FailureType> {
    #[allow(clippy::wildcard_enum_match_arm)]
    ImageReader::open(path)
        .map_err(|err| FailureType::IoError(err.to_string()))?
        .with_guessed_format()
        .map_err(|err| FailureType::IoError(err.to_string()))?
        .decode()
        .map_err(|err| match err {
            ImageError::Decoding(e) => FailureType::InvalidContent(e.to_string()),
            ImageError::Unsupported(e) => FailureType::UnsupportedFormat(e.to_string()),
            ImageError::IoError(e) => FailureType::IoError(e.to_string()),
            e => FailureType::InternalError(e.to_string()),
        })?;
    Ok(())
}

/// Handler: Use the `json` crate to do a basic well-formedness check
///
/// **TODO:** Decide on an API and some real-world test data to allow detecting potential
/// corruption in string variables using the UTF-8 subset of the plaintext handler's checks.
pub fn json(path: &Path) -> Result<(), FailureType> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let raw_data = fs::read_to_string(path).map_err(|err| match err.kind() {
        // If we can't String it, then report a validation error because JSON must be UTF-8
        io::ErrorKind::InvalidData => FailureType::InvalidContent(err.to_string()),
        // ...otherwise, report an OS-level error.
        _ => FailureType::IoError(err.to_string()),
    })?;

    // TODO: See if there's a Read-based API that could be used to reduce the memory footprint
    json::parse(&raw_data).map_err(|err| FailureType::InvalidContent(err.to_string()))?;
    Ok(())
}

/// Handler: Use the `toml` crate to do a basic well-formedness check
///
/// **TODO:** Decide on an API and some real-world test data to allow detecting potential
/// corruption in string variables using the UTF-8 subset of the plaintext handler's checks.
pub fn toml(path: &Path) -> Result<(), FailureType> {
    #[allow(clippy::wildcard_enum_match_arm)]
    let raw_data = fs::read_to_string(path).map_err(|err| match err.kind() {
        // If we can't String it, then report a validation error because JSON must be UTF-8
        io::ErrorKind::InvalidData => FailureType::InvalidContent(err.to_string()),
        // ...otherwise, report an OS-level error.
        _ => FailureType::IoError(err.to_string()),
    })?;

    // TODO: See if there's a Read-based API that could be used to reduce the memory footprint
    raw_data
        .parse::<toml_edit::Item>()
        .map_err(|err| FailureType::InvalidContent(err.to_string()))?;
    Ok(())
}

/// Handler: Use the `zip` crate to validate Zip files which use STORE or DEFLATE compression
///
/// **TODO:** Decide on the best API for selecting whether this should operate recursively to
/// validate files that it must extract anyway to check their CRCs.
///
/// (As a means to detect corruption that occurred before the archive was generated.)
pub fn zip(path: &Path) -> Result<(), FailureType> {
    /// Helper for `?` use pending the availability of `try` blocks in stable channel
    fn zip_inner(reader: &File) -> ZipResult<()> {
        let mut zip = ZipArchive::new(reader)?;
        for i in 0..zip.len() {
            exhaust_reader(zip.by_index(i)?)?; // Trigger CRC32 validation
        }
        Ok(())
    }

    let reader = File::open(path).map_err(|e| FailureType::IoError(e.to_string()))?;
    zip_inner(&reader).map_err(|err| match err {
        ZipError::Io(e) => FailureType::IoError(e.to_string()),
        ZipError::InvalidArchive(e) => FailureType::InvalidContent(e.to_string()),
        ZipError::UnsupportedArchive(e) => FailureType::UnsupportedFormat(e.to_string()),
        ZipError::FileNotFound => FailureType::InternalError(
            "'file not found' when reading Zip file by bounded index".to_string(),
        ),
    })?;
    Ok(())
}
