/*! Application-specific logic lives here */

// Parts Copyright 2017-2020, Stephan Sokolow

// Standard library imports
use std::path::PathBuf;

// 3rd-party crate imports
use anyhow::Result;
use clap::Parser;
use ignore::WalkBuilder;

use log::{debug, error, info, trace, warn};

// Local Imports
use crate::builtin_handlers::ALL as BUILTIN_HANDLERS;
use crate::config;
use crate::helpers::{BoilerplateOpts, HELP_TEMPLATE};
use crate::validators::path_input_file_or_dir;

/// The verbosity level when no `-q` or `-v` arguments are given, with `0` being `-q`
pub const DEFAULT_VERBOSITY: usize = 2;

/// The contents of the default configuration file that is used if nothing else is found
pub const DEFAULT_CONFIG: &str = include_str!("../../verifiers.toml");

/// Command-line argument schema
#[derive(Parser, Debug)]
#[clap(template = HELP_TEMPLATE,
       about = "A simple tool to recursively walk a set of paths and report corrupted files.",
       version,
       long_about = None)]
pub struct CliOpts {
    #[allow(clippy::missing_docs_in_private_items)] // TODO: Check if doccing still breaks stuff
    #[clap(flatten)]
    pub boilerplate: BoilerplateOpts,

    /// File(s) to use as input
    ///
    /// **TODO:** Restore use of `path_input_file_or_dir` validator
    #[clap(value_parser)]
    inpath: Vec<PathBuf>,

    /// Just quickly identify files that have no checker registered
    #[clap(long)]
    list_unrecognized: bool,

    /// Just list the built-in handlers which are available for use in the configuration file
    #[clap(long)]
    list_builtins: bool,
}

/// The actual `main()`
pub fn main(mut opts: CliOpts) -> Result<()> {
    if opts.list_builtins {
        for (id, (description, _)) in BUILTIN_HANDLERS.iter() {
            println!("{:10}\t{}", id, description);
        }
        return Ok(());
    }

    // TODO: Support reading a custom config before using the embedded one
    let config = config::parse(DEFAULT_CONFIG, &|x| BUILTIN_HANDLERS.contains_key(x))?;

    // XXX: Fix this once https://github.com/BurntSushi/ripgrep/issues/1761 is resolved.
    if let Some(path1) = opts.inpath.pop() {
        let mut builder = WalkBuilder::new(path1);
        builder.standard_filters(false);
        for ignore_pat in config.overrides.iter().filter(|x| x.ignore) {
            // TODO: Integration test the proper handling of ignores
            builder.add_custom_ignore_filename(&ignore_pat.path);
        }
        // TODO: Allow the standard filters to be toggled individually in the config file or via
        //       command-line arguments
        // TODO: Support all WalkBuilder arguments that don't make sense in the config file as
        //       command-line options.
        for path in opts.inpath {
            builder.add(path);
        }
        for result in builder.build() {
            // TODO: Have an internal validator (which can be turned off) which runs in addition to
            // the regular check and just looks for Win32-incompatible filenames.
            error!("TODO: Implement processing of {:?}", result?);
        }
    }

    Ok(())
}
