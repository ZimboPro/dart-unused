use dart_unused::{cli::Options, get_unreferenced_files};
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "dart-unused",
    about = "Check for unreferenced files in a Dart project",
    long_about = "Check for unreferenced files in a Dart project. This tool checks for unreferenced assets, dependencies, and dart files in a Dart project by default. You can also remove unreferenced files by using the --remove flag. You can specify what to check by using the flags --assets, --deps, and --dart."
)]
pub struct Args {
    #[arg(short, long, help = "Path to the Dart project")]
    pub path: PathBuf,
    #[arg(long, help = "Remove all unreferenced items discovered")]
    pub remove: bool,
    #[arg(short, long, help = "Check for unreferenced assets")]
    pub assets: bool,
    #[arg(short, long, help = "Check for unreferenced dependencies")]
    pub deps: bool,
    // #[arg(long, help = "Check for unreferenced dart files")]
    // pub dart: bool,
    #[arg(short, long, help = "Check for unused arb file(s) entries")]
    pub labels: bool,
    #[arg(long, help = "List items registered in locator but not used")]
    pub loc: bool,
    #[arg(short, long, help = "Enable verbose logging")]
    pub verbose: bool,
    // #[arg(long, short)]
    // pub format: bool,
    // #[arg(long, short)]
    // pub warn: bool,
    // #[arg(long, short, help = "Output the results to a file")]
    // pub output: bool,
}

impl From<Args> for Options {
    fn from(val: Args) -> Self {
        Self {
            assets: val.assets,
            deps: val.deps,
            labels: val.labels,
            loc: val.loc,
            path: val.path,
            remove: val.remove,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .build();
    let args = Args::parse();
    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    TermLogger::init(log_level, config, TerminalMode::Mixed, ColorChoice::Auto)?;
    get_unreferenced_files(args.into())
}
