use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Options {
    pub path: PathBuf,
    pub remove: bool,
    pub assets: bool,
    pub deps: bool,
    pub entries: Vec<PathBuf>,
    // pub dart: bool,
    pub labels: bool,
    pub loc: bool,
    // pub format: bool,
    // pub warn: bool,
    // pub output: bool,
}
