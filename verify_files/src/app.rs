/*! Application-specific logic lives here */

// Parts Copyright 2017-2020, Stephan Sokolow

// Standard library imports
use std::path::PathBuf;

// 3rd-party crate imports
use anyhow::Result;
use structopt::StructOpt;

use log::{debug, error, info, trace, warn};

// Local Imports
use crate::{config, processors};
use crate::helpers::{BoilerplateOpts, HELP_TEMPLATE};
use crate::validators::path_input_file_or_dir;

/// The verbosity level when no `-q` or `-v` arguments are given, with `0` being `-q`
pub const DEFAULT_VERBOSITY: u64 = 2;

/// The contents of the default configuration file that is used if nothing else is found
pub const DEFAULT_CONFIG: &str = include_str!("../../verifiers.toml");

/// Command-line argument schema
#[derive(StructOpt, Debug)]
#[structopt(template = HELP_TEMPLATE,
            about = "A simple tool to recursively walk a set of paths and report corrupted files.",
            global_setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct CliOpts {
    #[allow(clippy::missing_docs_in_private_items)] // StructOpt compile-time errors if we doc this
    #[structopt(flatten)]
    pub boilerplate: BoilerplateOpts,

    /// File(s) to use as input
    #[structopt(parse(from_os_str), validator_os = path_input_file_or_dir)]
    inpath: Vec<PathBuf>,

    /// Just quickly identify files that have no checker registered
    #[structopt(long)]
    list_unrecognized: bool,
}

/// The actual `main()`
pub fn main(opts: CliOpts) -> Result<()> {
    // TODO: Support reading a custom config before using the embedded one
    let test = config::parse(DEFAULT_CONFIG)?;
    println!("{:#?}", test);

    for inpath in opts.inpath {
        todo!("Implement application logic")
    }

    Ok(())
}

// Tests go below the code where they'll be out of the way when not the target of attention
#[cfg(test)]
mod tests {
    use super::CliOpts;

    // TODO: Unit test to verify that the doc comments on `CliOpts` or `BoilerplateOpts` aren't
    // overriding the intended about string.

    #[test]
    /// Test something
    fn test_something() {
        // TODO: Test something
    }
}
