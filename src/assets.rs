use std::{collections::HashSet, path::PathBuf};

use glob::glob;

use ouroboros::self_referencing;

#[self_referencing]
#[derive(Debug, PartialEq, Eq)]
pub(super) struct OsStringWithStr {
    pub(super) os: std::ffi::OsString,

    #[borrows(os)]
    pub(super) s: &'this str,
}

pub(crate) fn get_assets(
    enabled: bool,
    asset_paths: &Vec<PathBuf>,
    ignored_assets: &Vec<String>,
) -> anyhow::Result<Vec<OsStringWithStr>> {
    if enabled {
        let registered_assets = get_registered_assets(asset_paths)?;
        let registered_assets = remove_ignored_assets(registered_assets, ignored_assets)?;
        let mut assets = Vec::with_capacity(registered_assets.len());
        for asset in registered_assets.iter() {
            let p = asset.file_name().unwrap().to_owned();

            let v = OsStringWithStrBuilder {
                os: p,
                s_builder: |os| os.to_str().unwrap(),
            }
            .build();
            assets.push(v);
        }
        Ok(assets)
    } else {
        Ok(Vec::new())
    }
}

pub fn get_registered_assets(asset_paths: &Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    let mut assets: HashSet<PathBuf> = HashSet::new();
    for asset in asset_paths {
        let path = PathBuf::from(asset);
        if path.exists() {
            if path.is_file() {
                assets.insert(path);
            } else if path.is_dir() {
                let pattern = format!("{}/**", asset.to_str().unwrap());
                let items = glob(&pattern)
                    .expect("Failed to read glob pattern")
                    .flatten()
                    .collect::<Vec<_>>();
                for entry in items {
                    if entry.is_file() {
                        assets.insert(entry);
                    }
                }
            }
        }
    }
    Ok(assets.into_iter().collect())
}

pub fn get_all_items_in_asset_dir(asset_paths: &Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    let mut assets: HashSet<PathBuf> = HashSet::new();
    for asset in asset_paths {
        let path = PathBuf::from(asset);
        if path.exists() {
            if path.is_file() {
                assets.insert(path);
            } else if path.is_dir() {
                let pattern = format!("{}/**/*", asset.to_str().unwrap());
                let items = glob(&pattern)
                    .expect("Failed to read glob pattern")
                    .flatten()
                    .collect::<Vec<_>>();
                for entry in items {
                    if entry.is_file() {
                        assets.insert(entry);
                    }
                }
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
