//! Validator functions suitable for use with [clap](https://lib.rs/crates/clap) and
//! [structopt](https://lib.rs/crates/structopt).

// Copyright 2017-2020, Stephan Sokolow

use std::ffi::OsString;
use std::path::Path;

use faccess::{AccessMode, PathExt as _};

/// Test that the given path is a file or directory that it *should* be possible to read from.
///
/// (That either [`path_input_file`] or [`path_input_dir`] passes)
///
/// ## Use For:
///  * Input arguments which may be either a file or a directory to traverse to find files, which
///    don't need to be writable but which cannot be read from standard input. (For example,
///    because a quirk of an underlying API or subprocess precludes it.)
///
/// ## Relevant Conventions:
///  * **Prefer [`path_input_any`].**
///    Commands should support taking input via `stdin` whenever feasible.
///  * Use `-r` or `-R` to request recursive traversal when it is not the default.
///    [\[1\]](http://www.catb.org/esr/writings/taoup/html/ch10s05.html)
///
/// ## Cautions:
///  * Never assume a file or directory's permissions will remain unchanged between the time you
///    check them and the time you attempt to use them.
pub fn path_input_file_or_dir<P: AsRef<Path> + ?Sized>(value: &P) -> Result<(), OsString> {
    let path = value.as_ref();

    if path_input_file(value).is_ok() || path_input_dir(value).is_ok() {
        return Ok(());
    }

    Err(format!("Not a readable file or directory: {}", path.display()).into())
}

/// Test that the given path is a directory that it *should* be possible to read files from
///
/// ## Use For:
///  * Input arguments which must be a path to a directory, but not a file.
///
/// ## Relevant Conventions:
///  * **Prefer [`path_input_file_or_dir`] or [`path_input_any`].**
///    Commands which traverse directories to find files should support paths pointing directly at
///    files where feasible.
///  * Use `-r` or `-R` to request recursive traversal when it is not the default.
///    [\[1\]](http://www.catb.org/esr/writings/taoup/html/ch10s05.html)
///
/// ## Cautions:
///  * Never assume a directory's permissions will remain unchanged between the time you check them
///    and the time you attempt to use them.
pub fn path_input_dir<P: AsRef<Path> + ?Sized>(value: &P) -> Result<(), OsString> {
    let path = value.as_ref();

    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()).into());
    }

    if path.access(AccessMode::READ | AccessMode::EXECUTE).is_ok() {
        return Ok(());
    }

    Err(format!("Would be unable to read and traverse destination directory: {}", path.display())
        .into())
}

/// The given path is a file that can be opened for reading
///
/// ## Use For:
///  * Input file paths
///
/// ## Relevant Conventions:
///  * **Prefer [`path_input_file_or_stdin`].**
///    Commands should support taking input via `stdin` whenever feasible.
///  * If specifying an input file via an option flag, use `-f` as the name of the flag.
///    [\[1\]](http://www.catb.org/esr/writings/taoup/html/ch10s05.html)
///  * Prefer taking input paths as positional arguments and, if feasible, allow an arbitrary
///    number of input arguments. This allows easy use of shell globs.
///
/// ## Cautions:
///  * Never assume a file's permissions will remain unchanged between the time you check them
///    and the time you attempt to use them.
///  * As a more reliable validity check, you are advised to open a handle to the file in question
///    as early in your program's operation as possible, use it for all your interactions with the
///    file, and keep it open until you are finished. This will both verify its validity and
///    minimize the window in which another process could render the path invalid.
#[rustfmt::skip]
pub fn path_input_file<P: AsRef<Path> + ?Sized>(value: &P)
        -> std::result::Result<(), OsString> {
    let path = value.as_ref();

    if path.is_dir() {
        return Err(format!("{}: Input path must be a file, not a directory",
                           path.display()).into());
    }

    if path.readable() {
       return Ok(())
    }
    Err(format!("Would be unable to write to destination file: {}", path.display()).into())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::wildcard_imports, clippy::panic, clippy::expect_used)] // OK for tests

    use super::*;
    use std::ffi::OsStr;

    #[cfg(unix)]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(windows)]
    use std::os::windows::ffi::OsStringExt;

    // TODO: Come up with a tidier way to handle these tests that's less likely to make a human's
    //       eyes glaze over when trying to audit it. (Maybe a CSV file where rows are test
    //       strings, columns are validators, and cells are whether they should pass? Then,
    //       auditing could be done in something which visualizes it more, like LibreOffice)


    #[test]
    #[cfg(unix)]
    #[rustfmt::skip]
    fn path_input_file_or_dir_basic_functionality() {
        assert!(path_input_file_or_dir(OsStr::new("-")).is_err());                      // stdin
        assert!(path_input_file_or_dir(OsStr::new("/tmp")).is_ok());                    // OK Fldr
        assert!(path_input_file_or_dir(OsStr::new("/dev/null")).is_ok());               // OK File
        assert!(path_input_file_or_dir(OsStr::new("/etc/shadow")).is_err());            // Dny File
        assert!(path_input_file_or_dir(OsStr::new("/etc/ssl/private")).is_err());       // Dny Fldr
        assert!(path_input_file_or_dir(OsStr::new("/nonexistant_test_path")).is_err()); // Missing
        assert!(path_input_file_or_dir(OsStr::new("/tmp\0with\0null")).is_err());       // Bad CStr
        // TODO: Not-already-canonicalized paths (eg. relative paths)

        assert!(path_input_dir(OsStr::from_bytes(b"/not\xffutf8")).is_err());   // Invalid UTF-8
        // TODO: Non-UTF8 path that actually does exist and is writable
    }

    #[test]
    #[cfg(windows)]
    fn path_input_file_or_dir_basic_functionality() {
        unimplemented!("TODO: Implement unit test for Windows version of path_input_dir");
    }

    #[test]
    #[cfg(unix)]
    #[rustfmt::skip]
    fn path_input_dir_basic_functionality() {
        assert!(path_input_dir(OsStr::new("/")).is_ok());                       // Root
        assert!(path_input_dir(OsStr::new("/tmp")).is_ok());                    // OK Folder
        assert!(path_input_dir(OsStr::new("/dev/null")).is_err());              // OK File
        assert!(path_input_dir(OsStr::new("/etc/shadow")).is_err());            // Denied File
        assert!(path_input_dir(OsStr::new("/etc/ssl/private")).is_err());       // Denied Folder
        assert!(path_input_dir(OsStr::new("/nonexistant_test_path")).is_err()); // Missing Path
        assert!(path_input_dir(OsStr::new("/tmp\0with\0null")).is_err());       // Invalid CString
        // TODO: Not-already-canonicalized paths (eg. relative paths)

        assert!(path_input_dir(OsStr::from_bytes(b"/not\xffutf8")).is_err());   // Invalid UTF-8
        // TODO: Non-UTF8 path that actually does exist and is writable
    }

    #[test]
    #[cfg(windows)]
    fn path_input_dir_basic_functionality() {
        unimplemented!("TODO: Implement unit test for Windows version of path_input_dir");
    }

    // ---- path_input_file ----

    #[test]
    fn path_input_file_stdin_test() {
        assert!(path_input_file(OsStr::new("-")).is_err());
    }

    #[cfg(unix)]
    #[test]
    #[rustfmt::skip]
    fn path_input_file_basic_functionality() {
        for func in &[path_input_file] {
            // Existing paths
            assert!(func(OsStr::new("/bin/sh")).is_ok());                 // OK File
            assert!(func(OsStr::new("/bin/../etc/.././bin/sh")).is_ok()); // Non-canonicalized
            assert!(func(OsStr::new("/../../../../bin/sh")).is_ok());     // Above root

            // Inaccessible, nonexistent, or invalid paths
            assert!(func(OsStr::new("")).is_err());                       // Empty String
            assert!(func(OsStr::new("/")).is_err());                      // OK Folder
            assert!(func(OsStr::new("/etc/shadow")).is_err());            // Denied File
            assert!(func(OsStr::new("/etc/ssl/private")).is_err());       // Denied Folder
            assert!(func(OsStr::new("/nonexistant_test_path")).is_err()); // Missing Path
            assert!(func(OsStr::new("/null\0containing")).is_err());      // Invalid CString
        }
    }

    #[cfg(windows)]
    #[test]
    fn path_input_file_basic_functionality() {
        unimplemented!("TODO: Pick some appropriate equivalent test paths for Windows");
    }

    #[cfg(unix)]
    #[test]
    #[rustfmt::skip]
    fn path_input_file_invalid_utf8() {
        for func in &[path_input_file] {
            assert!(func(OsStr::from_bytes(b"/not\xffutf8")).is_err()); // Invalid UTF-8
            // TODO: Non-UTF8 path that actually IS valid
        }
    }
    #[cfg(windows)]
    #[test]
    #[rustfmt::skip]
    fn path_input_file_unpaired_surrogates() {
        for func in &[path_input_file] {
            assert!(path_input_file(&OsString::from_wide(
                &['C' as u16, ':' as u16, '\\' as u16, 0xd800])).is_err());
            // TODO: Unpaired surrogate path that actually IS valid
        }
    }
}
