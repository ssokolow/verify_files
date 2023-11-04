//! A simple tool for detecting corruption in files to the greatest extent feasible without
//! reliable access to MD5, SHA1, PAR2, or other external hash/ECC files.

// This source file provided by
// [rust-cli-boilerplate](https://github.com/ssokolow/rust-cli-boilerplate)
//
// Copyright 2017-2020, Stephan Sokolow

// Make rustc's built-in lints more strict and set clippy into a whitelist-based configuration so
// we see new lints as they get written, then opt out of ones we have seen and don't want
#![warn(warnings, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(clippy::float_arithmetic, clippy::implicit_return, clippy::needless_return)]
#![allow(clippy::blanket_clippy_restriction_lints)]
#![forbid(unsafe_code)] // Enforce my policy of only allowing it in my own code as a last resort

// 3rd-party imports
use anyhow::{Context, Result};
use clap::Parser;

// Local imports
mod app;
mod builtin_handlers;
mod config;
mod validators;

/// Boilerplate to parse command-line arguments, set up logging, and handle bubbled-up `Error`s.
///
/// See `app::main` for the application-specific logic.
fn main() -> Result<()> {
    // Parse command-line arguments (exiting on parse error, --version, or --help)
    let opts = app::CliOpts::parse();

    stderrlog::new()
        .module(module_path!())
        .verbosity(opts.verbose.log_level_filter())
        .timestamp(opts.timestamp.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .context("Failed to initialize logging output")?;

    // TODO: Re-enable completion support
    app::main(opts)
}

// Tests go below the code where they'll be out of the way when not the target of attention
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    /// Run [`App::debug_assert`](clap::builder::App::debug_assert) checks
    fn verify_cli() {
        use clap::CommandFactory;
        app::CliOpts::command().debug_assert()
    }
}

// vim: set sw=4 sts=4 expandtab :
