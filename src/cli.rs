use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    pub file: Option<PathBuf>,

    /// canonical hex+ASCII display
    ///
    /// This option is currently the default and thus ignored
    #[arg(short = 'C', long)]
    pub canonical: bool,

    /// Interpret only length bytes of input.
    #[arg(short = 'n', long, default_value_t = usize::MAX)]
    pub length: usize,

    /// Skip this many bytes of input
    /// Note: if this value is not a multiple of 16, then it will not have the _exact_ same output
    /// as hexdump, though it will still be correct.
    #[arg(short, long, default_value_t = 0)]
    pub skip: usize,

    /// The -v option causes hexdump to display all input data. Without the -v option, any number of groups of output lines which would be identical to the immediately
    /// preceding group of output lines (except for the input offsets), are replaced with a line comprised of a single asterisk.
    #[arg(short = 'v', long)]
    pub no_squeeze: bool,

    /// When to use terminal colours (always, auto, never). default is never
    #[arg(long, default_value = "never")]
    pub color: String,
}
