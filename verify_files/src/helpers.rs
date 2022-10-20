/*! Functions and templates which can be imported by `app.rs` to save effort */
// Copyright 2017-2019, Stephan Sokolow

use clap::Parser;

/// Modified version of Clap's default template for proper
/// [help2man](https://www.gnu.org/software/help2man/) compatibility
///
/// Used as a workaround for:
/// 1. Clap's default template interfering with `help2man`'s proper function
///    ([clap-rs/clap/#1432](https://github.com/clap-rs/clap/issues/1432))
/// 2. Workarounds involving injecting `\n` into the description breaking help output if used
///    on subcommand descriptions.
pub const HELP_TEMPLATE: &str = "{bin} {version}

{about}

USAGE:
    {usage}

{all-args}
";

#[allow(clippy::missing_docs_in_private_items)]
// Can't doc-comment until TeXitoi/structopt#333
// Options used by boilerplate code in `main.rs`
//
// FIXME: Report that StructOpt trips Clippy's `cast_possible_truncation` lint unless I use
//        `u64` for my `from_occurrences` inputs, which is a ridiculous state of things.
#[derive(Parser, Debug)]
#[clap(rename_all = "kebab-case")]
pub struct BoilerplateOpts {
    /// Decrease verbosity (-q, -qq, -qqq, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub quiet: usize,

    /// Increase verbosity (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,

    /// Display timestamps on log messages (sec, ms, ns, none)
    #[clap(short, long, value_name = "resolution")]
    pub timestamp: Option<stderrlog::Timestamp>,
}
