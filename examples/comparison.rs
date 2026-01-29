// A small test to compare the performance of using Regex with word boundaries and iterating from the bytes
// directly and just checking the previous and the next byte for non-word characters.

use clap::Parser;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Parser, Clone)]
#[clap(
    name = "dart-unused-comparision",
    about = "Compare performance of different string search methods",
    long_about = "Compare performance of different string search methods in Rust"
)]
pub struct Args {
    #[arg(short, long, help = "Path to the file to search")]
    pub file: PathBuf,
    #[arg(short, long, help = "String to search for")]
    pub search: String,
    #[arg(short, long, help = "Number of iterations to run")]
    pub iterations: usize,
    #[arg(short, long, help = "Enable verbose logging")]
    pub verbose: bool,
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
    TermLogger::init(log_level, config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();

    let contents = std::fs::read_to_string(&args.file)
        .unwrap_or_else(|_| panic!("Failed to read file: {:?}", args.file));
    let terms: Vec<&str> = args.search.split(',').collect();
    // Prepare the regex

    let regex: Vec<regex::Regex> = terms
        .iter()
        .map(|x| {
            let pattern = format!(r"\b({})\b", regex::escape(x));
            regex::Regex::new(&pattern).unwrap()
        })
        .collect();

    // Benchmark Regex search
    let start_regex = Instant::now();
    let mut cnt = 0;
    for _ in 0..args.iterations {
        cnt = 0;
        for re in regex.iter() {
            for _ in re.find_iter(&contents) {
                cnt += 1;
            }
        }
    }
    let duration_regex = start_regex.elapsed();
    println!(
        "Regex search took: {:?} for {} iterations: {} matches found",
        duration_regex, args.iterations, cnt
    );

    // Using memchr

    // Benchmark memchr search

    let start_memchr = Instant::now();
    let mut cnt = 0;
    for _ in 0..args.iterations {
        cnt = 0;
        for term in terms.iter() {
            for s in memchr::memmem::find_iter(&contents.as_bytes(), term.as_bytes()) {
                if s == 0
                    || (!contents.as_bytes()[s - 1].is_ascii_alphanumeric()
                        && contents.as_bytes()[s - 1] != b'_')
                {
                    if s + term.len() == contents.len()
                        || (!contents.as_bytes()[s + term.len()].is_ascii_alphanumeric()
                            && contents.as_bytes()[s + term.len()] != b'_')
                    {
                        cnt += 1;
                    }
                }
            }
        }
    }
    let duration_memchr = start_memchr.elapsed();
    println!(
        "memchr search took: {:?} for {} iterations: {} matches found",
        duration_memchr, args.iterations, cnt
    );

    // Benchmark memchr search

    let finders = terms
        .iter()
        .map(|term| memchr::memmem::Finder::new(term.as_bytes()))
        .collect::<Vec<_>>();
    let start_memchr = Instant::now();
    let mut cnt = 0;
    for _ in 0..args.iterations {
        cnt = 0;
        for term in finders.iter() {
            for s in term.find_iter(&contents.as_bytes()) {
                if s == 0
                    || (!contents.as_bytes()[s - 1].is_ascii_alphanumeric()
                        && contents.as_bytes()[s - 1] != b'_')
                {
                    if s + term.needle().len() == contents.len()
                        || (!contents.as_bytes()[s + term.needle().len()].is_ascii_alphanumeric()
                            && contents.as_bytes()[s + term.needle().len()] != b'_')
                    {
                        cnt += 1;
                    }
                }
            }
        }
    }
    let duration_memchr = start_memchr.elapsed();
    println!(
        "memchr finder search took: {:?} for {} iterations: {} matches found",
        duration_memchr, args.iterations, cnt
    );

    // Using aho-corasick
    let ac = aho_corasick::AhoCorasick::new(&terms).unwrap();
    let start_aho = Instant::now();
    let mut cnt = 0;
    for _ in 0..args.iterations {
        cnt = 0;
        for s in ac.find_iter(&contents) {
            if s.start() == 0
                || (!contents.as_bytes()[s.start() - 1].is_ascii_alphanumeric()
                    && contents.as_bytes()[s.start() - 1] != b'_')
            {
                if s.end() == contents.len()
                    || (!contents.as_bytes()[s.end()].is_ascii_alphanumeric()
                        && contents.as_bytes()[s.end()] != b'_')
                {
                    cnt += 1;
                }
            }
        }
    }
    let duration_aho = start_aho.elapsed();
    println!(
        "Aho-Corasick search took: {:?} for {} iterations: {} matches found",
        duration_aho, args.iterations, cnt
    );
    Ok(())
}
