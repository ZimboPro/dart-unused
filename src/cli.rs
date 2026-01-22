use std::path::PathBuf;

pub struct Options {
    pub path: PathBuf,
    pub remove: bool,
    pub assets: bool,
    pub deps: bool,
    // pub dart: bool,
    pub labels: bool,
    pub loc: bool,
    // pub format: bool,
    // pub warn: bool,
    // pub output: bool,
}
