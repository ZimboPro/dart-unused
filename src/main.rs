use dart_unused::{cli::Options, get_unreferenced_files};
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError, set_boxed_logger, set_max_level};

use std::{io::Write, path::PathBuf};

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
    #[arg(short, long, short, help = "Output the results to a file")]
    pub output: bool,
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

struct BufferedWriteLogger {
    level: LevelFilter,
    buffer: std::sync::Mutex<Vec<u8>>,
    file: Option<PathBuf>,
}

impl BufferedWriteLogger {
    pub fn init(log_level: LevelFilter, file: Option<PathBuf>) -> Result<(), SetLoggerError> {
        set_max_level(log_level);
        set_boxed_logger(BufferedWriteLogger::new(log_level, file))
    }

    pub fn new(log_level: LevelFilter, file: Option<PathBuf>) -> Box<BufferedWriteLogger> {
        Box::new(BufferedWriteLogger {
            level: log_level,
            file,
            buffer: std::sync::Mutex::new(Vec::new()),
        })
    }
}

impl Log for BufferedWriteLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            self.buffer
                .lock()
                .unwrap()
                .extend_from_slice(format!("{}\n", record.args()).as_bytes());
        }
    }

    fn flush(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        if !buffer.is_empty() {
            std::io::stdout().write_all(&buffer).unwrap();
            std::io::stdout().flush().unwrap();
            if let Some(file) = &self.file {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file)
                    .unwrap()
                    .write_all(&buffer)
                    .unwrap();
            }
            buffer.clear();
        }
    }
}

impl Drop for BufferedWriteLogger {
    fn drop(&mut self) {
        self.flush();
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let log_level = if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    if args.output {
        BufferedWriteLogger::init(log_level, Some("dart-unused.log".into()))?;
    } else {
        BufferedWriteLogger::init(log_level, None)?;
    }
    get_unreferenced_files(args.into())
}
