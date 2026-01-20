use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub format_ignore: Vec<String>,
    #[serde(default)]
    pub assets: Assets,
    #[serde(default)]
    pub deps: Deps,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Assets {
    pub ignore: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Deps {
    pub ignore: Vec<String>,
}
