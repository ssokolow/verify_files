/*! Application-specific logic lives here */

// Parts Copyright 2017-2020, Stephan Sokolow

// Standard library imports
use std::path::PathBuf;

// 3rd-party crate imports
use anyhow::Result;
use clap::{
    builder::styling::{AnsiColor, Styles},
    //builder::{PathBufValueParser, TypedValueParser},
    Parser,
};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use ignore::WalkBuilder;

use log::{debug, error, info, trace, warn};

// Local Imports
use crate::builtin_handlers::ALL as BUILTIN_HANDLERS;
use crate::config;
use crate::validators::path_input_file_or_dir;

/// The contents of the default configuration file that is used if nothing else is found
pub const DEFAULT_CONFIG: &str = include_str!("../../verifiers.toml");

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

/// Command-line argument schema
#[derive(Parser, Debug)]
#[clap(about = "A simple tool to recursively walk a set of paths and report corrupted files.",
       version,
       long_about = None,
       styles = styles())]
pub struct CliOpts {
    #[command(flatten)]
    pub verbose: Verbosity<WarnLevel>,

    /// Display timestamps on log messages (sec, ms, ns, none)
    #[arg(short, long, value_name = "resolution")]
    pub timestamp: Option<stderrlog::Timestamp>,

    /// File(s) to use as input
    // **TODO:** Restore use of `path_input_file_or_dir` validator
    inpath: Vec<PathBuf>,

    /// Just quickly identify files that have no checker registered
    #[arg(long)]
    list_unrecognized: bool,

    /// Just list the built-in handlers which are available for use in the configuration file
    #[arg(long)]
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
