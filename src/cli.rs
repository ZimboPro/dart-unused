use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "dart-unused",
    about = "Check for unreferenced files in a Dart project",
    long_about = "Check for unreferenced files in a Dart project. This tool checks for unreferenced assets, dependencies, and dart files in a Dart project by default. You can also remove unreferenced files by using the --remove flag. You can specify what to check by using the flags --assets, --deps, and --dart."
)]
pub struct CLI {
    #[arg(short, long, help = "Path to the Dart project")]
    pub path: PathBuf,
    // #[arg(short, long, help = "Path to the Dart package arb file")]
    // package: PathBuf,
    #[arg(long, help = "Remove unreferenced items")]
    pub remove: bool,
    #[arg(short, long, help = "Check for unreferenced assets")]
    pub assets: bool,
    #[arg(short, long, help = "Check for unreferenced dependencies")]
    pub deps: bool,
    #[arg(long, help = "Check for unreferenced dart files")]
    pub dart: bool,
    #[arg(short, long, help = "Check for unused arb file(s) entries")]
    pub labels: bool,
    #[arg(long, help = "List items registered in locator but not used")]
    pub loc: bool,
    #[arg(long, short)]
    pub format: bool,
    #[arg(long, short)]
    pub warn: bool,
    #[arg(long, short, help = "Output the results to a file")]
    pub output: bool,
}
