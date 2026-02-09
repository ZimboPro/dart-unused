use glob::glob;
use std::hash::Hash;
use std::{collections::HashSet, path::PathBuf};

use log::{debug, info, warn};

#[derive(Debug, Clone)]
pub(super) struct AssetItem {
    pub(super) path: PathBuf,

    pub(super) file_name: String,
}

impl PartialEq for AssetItem {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Hash for AssetItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl Eq for AssetItem {}

impl PartialOrd for AssetItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AssetItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl AssetItem {
    pub(super) fn new(path: PathBuf) -> Self {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
        AssetItem { path, file_name }
    }
}

pub(crate) fn get_assets(
    registered_assets: Vec<PathBuf>,
    ignored_assets: &Vec<String>,
) -> anyhow::Result<Vec<AssetItem>> {
    info!("Finding registered assets");
    debug!("{} registered assets", registered_assets.len());
    let registered_assets = remove_ignored_assets(registered_assets, ignored_assets)?;
    debug!(
        "{} registered assets after removing ignored assets",
        registered_assets.len()
    );
    let mut assets = Vec::with_capacity(registered_assets.len());
    registered_assets
        .into_iter()
        .for_each(|asset_path| assets.push(AssetItem::new(asset_path)));
    Ok(assets)
}

pub fn get_registered_assets(asset_paths: &Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    let mut assets: HashSet<PathBuf> = HashSet::new();
    for asset in asset_paths {
        debug!("Looking in {:?}", asset);
        let path = PathBuf::from(asset);
        if path.exists() {
            if path.is_file() {
                assets.insert(path);
            } else if path.is_dir() {
                let pattern = format!("{}/*", asset.to_str().unwrap());
                let items = glob(&pattern)
                    .expect("Failed to read glob pattern")
                    .flatten()
                    .collect::<Vec<_>>();
                for entry in items {
                    if entry.is_file() {
                        assets.insert(entry);
                    }
                }
            } else {
                warn!("Path {:?} does not exist", asset);
            }
        }
    }
    Ok(assets.into_iter().collect())
}

pub fn remove_ignored_assets(
    all_assets: Vec<PathBuf>,
    ignored_assets: &Vec<String>,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut ignored_set: HashSet<PathBuf> = HashSet::new();
    for ignored in ignored_assets {
        let path = PathBuf::from(ignored);
        if path.is_file() {
            ignored_set.insert(path);
        } else {
            let pattern = ignored.to_string();
            let items = glob(&pattern)
                .expect("Failed to read glob pattern")
                .flatten()
                .collect::<Vec<_>>();
            for entry in items {
                if entry.is_file() {
                    ignored_set.insert(entry);
                }
            }
        }
    }
    let filtered_assets: Vec<PathBuf> = all_assets
        .iter()
        .filter(|asset| !ignored_set.contains(*asset))
        .cloned()
        .collect();
    Ok(filtered_assets)
}
