use std::{
    collections::HashSet,
    hash::{Hash, RandomState},
    path::{Path, PathBuf},
};

use glob::glob;
use log::info;
use path_dedot::ParseDot;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub mod assets;
pub mod cli;
pub mod config;
pub mod localisation;
pub mod locator;
pub mod parser;
pub mod pubspec;
pub mod util;

use crate::{
    assets::{AssetItem, get_all_items_in_asset_dir, get_assets},
    localisation::all_localisation,
};

pub fn get_unreferenced_files(args: cli::Options) -> anyhow::Result<()> {
    let config: config::Config = if let Ok(s) = std::fs::read_to_string("unused.config.yaml") {
        serde_yaml2::from_str(&s).unwrap()
    } else {
        Default::default()
    };

    info!("Analyzing project at {:?}", args.path);
    util::set_current_dir(&args.path)?;
    info!("Current directory set to {:?}", std::env::current_dir()?);
    let pubspec = pubspec::get_package_details()?;
    let assets = if args.assets {
        get_assets(pubspec.flutter.get_assets(), &config.assets.ignore)?
    } else {
        Vec::new()
    };

    let registered_assets: HashSet<PathBuf> = assets.iter().map(|x| x.path.clone()).collect();
    // info!("{} assets registered", assets.len());
    let deps: Vec<String> = if args.deps {
        pubspec.dependencies.keys().cloned().collect()
    } else {
        Vec::new()
    };
    // TODO allow to set entry point
    localisation::set_class_name(&pubspec.flutter_intl.class_name)?;
    let main = PathBuf::from("lib/main.dart");
    // extracted_data.referenced_files.insert(main.clone());

    // extract_data(
    //     &main,
    //     &pubspec.name,
    //     &mut extracted_data,
    //     &mut deps,
    //     &mut assets,
    //     &args,
    // )?;

    let dart = glob("lib/**/*.dart").expect("Failed to read glob pattern");
    let dart: Vec<PathBuf> = dart.flatten().collect();
    // let mut assets_set: dashmap::DashSet<AssetItem> =
    //     dashmap::DashSet::from_iter(assets.into_iter());
    let mut assets_set: papaya::HashSet<AssetItem, RandomState> =
        papaya::HashSet::from_iter(assets.into_iter());

    let locator: papaya::HashMap<String, bool> = papaya::HashMap::with_capacity(dart.len() / 10);
    let labels: papaya::HashSet<String> = papaya::HashSet::with_capacity(dart.len() / 10);

    let results: dashmap::DashMap<PathBuf, Data> = dart
        .clone()
        .into_par_iter()
        .map(|path| {
            let items = extract_single_file(
                &path,
                &pubspec.name,
                &assets_set,
                args.clone(),
                &locator,
                &labels,
            );
            (
                path.clone(),
                Data {
                    path,
                    items: items.0,
                    assets: items.1,
                },
            )
        })
        .collect();

    let mut dart: HashSet<PathBuf> = dart.into_iter().collect();

    // Collect all the linked files from the entry file
    collapse_list(&main, &results, &mut dart, &mut assets_set);

    if !assets_set.is_empty() {
        let assets_set = assets_set.pin();
        let remaining_assets: Vec<PathBuf> = assets_set.iter().map(|x| x.path.clone()).collect();
        for asset in remaining_assets.iter().enumerate() {
            log::error!(
                "{}. Unreferenced registered assets: {:?}",
                asset.0 + 1,
                asset.1
            );
        }
        log::info!("");
        let mut all_assets: Vec<PathBuf> =
            get_all_items_in_asset_dir(&pubspec.flutter.get_asset_paths(), &config.assets.ignore)?;

        all_assets.retain(|x| !registered_assets.contains(x));

        if !all_assets.is_empty() {
            for asset in all_assets.iter().enumerate() {
                log::error!("{}. Unregistered asset: {:?}", asset.0 + 1, asset.1);
            }
            log::info!("");
        }
        if args.remove {
            for asset in all_assets.iter() {
                std::fs::remove_file(asset)?;
            }
        }
    }
    if args.deps {
        for dep in deps.iter().enumerate() {
            log::error!("{}. Unused dependencies: {:?}", dep.0 + 1, dep.1);
        }
        log::info!("");
    }

    if args.labels {
        // read arb files to get all localisation keys
        let mut all_localisation_keys: HashSet<String> = HashSet::with_capacity(10_000);
        let arb_files = glob("lib/l10n/*.arb").expect("Failed to read glob pattern");
        for arb in arb_files.flatten() {
            let contents = std::fs::read_to_string(&arb).expect("Failed to read arb file");
            let json: serde_json::Value =
                serde_json::from_str(&contents).expect("Failed to parse arb file");
            if let serde_json::Value::Object(map) = json {
                for (key, _) in map.iter() {
                    all_localisation_keys.insert(key.to_owned());
                }
            }
        }
        {
            let labels = labels.pin();
            all_localisation_keys.retain(|x| !labels.contains(x));
        }

        for label in all_localisation_keys.iter().enumerate() {
            log::error!(
                "{}. Unreferenced localisation key: {:?}",
                label.0 + 1,
                label.1
            );
        }
        log::info!("");
    }

    if args.loc {
        let mut locators = locator.pin();
        locators.retain(|_, v| !*v);
        for (ind, (k, _)) in locators.iter().enumerate() {
            log::error!("{}. Unused locator: {:?}", ind + 1, k);
        }
        log::info!("");
    }

    for file in dart.iter().enumerate() {
        log::error!("{} Unreferenced file: {:?}", file.0 + 1, file.1);
    }
    if args.remove {
        for file in dart.iter() {
            std::fs::remove_file(file)?;
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Data {
    path: std::path::PathBuf,
    items: Vec<ExtractedData>,
    assets: Vec<AssetItem>,
}

impl Hash for Data {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExtractedData {
    Path(PathBuf),
    Dep(String),
}

fn extract_single_file(
    file_path: &std::path::PathBuf,
    package_name: &str,
    assets: &papaya::HashSet<AssetItem, RandomState>,
    args: cli::Options,
    locator: &papaya::HashMap<String, bool>,
    labels: &papaya::HashSet<String>,
) -> (Vec<ExtractedData>, Vec<AssetItem>) {
    let mut files = Vec::new();
    let contents = std::fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Failed to read file: {:?}", file_path));
    for line in contents.lines() {
        if let Ok((_, dart)) = parser::dart_file(line) {
            match dart {
                parser::DartFile::Import(path) => {
                    // relative path imports
                    let file = path.replace("%20", " ");
                    let file = Path::new(&file);
                    let file = file_path.parent().unwrap().join(file);
                    files.push(ExtractedData::Path(file.parse_dot().unwrap().to_path_buf()));
                }
                parser::DartFile::Package(name, mut path) => {
                    // package imports
                    if name == package_name {
                        path.insert_str(0, "lib");
                        let path = path.replace("%20", " ");
                        let file = Path::new(&path);
                        files.push(ExtractedData::Path(file.to_path_buf()));
                    } else {
                        // referenced_packages.push(DartFile::Package(name, path));
                        // Remove deps used in referenced files
                        files.push(ExtractedData::Dep(name));
                    }
                }
                parser::DartFile::Part(value) => {
                    // part files
                    let mut file = file_path.clone();
                    file.set_file_name(value);
                    files.push(ExtractedData::Path(file));
                }
                parser::DartFile::Export(path) => {
                    let file = path.replace("%20", " ");
                    let file = Path::new(&file);
                    let file = file_path.parent().unwrap().join(file);
                    files.push(ExtractedData::Path(file.parse_dot().unwrap().to_path_buf()));
                }
            }
        }
    }

    let mut referenced_asset_files = HashSet::with_capacity(10);

    let mut assets = assets.pin();
    // First, collect all assets that match the content
    assets.iter().for_each(|asset| {
        if asset.file_name.is_match(&contents) {
            referenced_asset_files.insert(asset.clone());
        }
    });

    // Remove referenced assets from the set to speed up future checks
    if !referenced_asset_files.is_empty() {
        assets.retain(|asset| !referenced_asset_files.contains(asset));
    }

    if args.labels {
        let s = all_localisation(&contents);
        if let Ok((_, keys)) = s {
            let mut labels_referenced = labels.pin();
            for key in keys {
                labels_referenced.insert(key.to_owned());
            }
        }
    }

    if args.loc
        && let Ok((_, r)) = locator::locator(&contents)
    {
        let mut locators = locator.pin();
        for reg in r {
            match reg {
                locator::Locator::Register(s) => {
                    let _ = locators.get_or_insert(s, false);
                }
                locator::Locator::Get(s) => {
                    locators.insert(s, true);
                }
                _ => {}
            }
        }
    }
    (files, referenced_asset_files.into_iter().collect())
}

fn collapse_list(
    path: &PathBuf,
    files: &dashmap::DashMap<PathBuf, Data>,
    referenced: &mut HashSet<PathBuf>,
    assets: &mut papaya::HashSet<AssetItem, RandomState>,
) {
    let file = files.get(path).unwrap();
    for entry in &file.items {
        match entry {
            ExtractedData::Path(path_buf) => {
                if referenced.remove(path_buf) {
                    // File was in the list
                    collapse_list(path_buf, files, referenced, assets);
                }
            }
            ExtractedData::Dep(_) => {
                // TODO
            }
        }
    }
    let assets = assets.pin();

    for asset in &file.assets {
        // Assets are not in the dart list
        assets.remove(asset);
    }
}

// fn extract_data(
//     file_path: &std::path::PathBuf,
//     package_name: &str,
//     extracted_data: &mut ExtractData,
//     deps: &mut Vec<String>,
//     assets: &mut Vec<AssetItem>,
//     args: &cli::Options,
// ) -> anyhow::Result<()> {
//     let contents = std::fs::read_to_string(file_path)
//         .unwrap_or_else(|_| panic!("Failed to read file: {:?}", file_path));
//     for line in contents.lines() {
//         if let Ok((_, dart)) = parser::dart_file(line) {
//             match dart {
//                 parser::DartFile::Import(path) => {
//                     // relative path imports
//                     let file = path.replace("%20", " ");
//                     let file = Path::new(&file);
//                     let file = file_path.parent().unwrap().join(file);
//                     if !extracted_data
//                         .referenced_files
//                         .contains(&file.to_path_buf())
//                     {
//                         extracted_data
//                             .referenced_files
//                             .insert(file.parse_dot().unwrap().to_path_buf());
//                         extract_data(
//                             &file.parse_dot().unwrap().to_path_buf(),
//                             package_name,
//                             extracted_data,
//                             deps,
//                             assets,
//                             args,
//                         )?;
//                     }
//                 }
//                 parser::DartFile::Package(name, mut path) => {
//                     // package imports
//                     if name == package_name {
//                         path.insert_str(0, "lib");
//                         let path = path.replace("%20", " ");
//                         let file = Path::new(&path);
//                         if !extracted_data
//                             .referenced_files
//                             .contains(&file.to_path_buf())
//                         {
//                             extracted_data.referenced_files.insert(file.to_path_buf());
//                             extract_data(
//                                 &file.to_path_buf(),
//                                 package_name,
//                                 extracted_data,
//                                 deps,
//                                 assets,
//                                 args,
//                             )?;
//                         }
//                     } else {
//                         // referenced_packages.push(DartFile::Package(name, path));
//                         // Remove deps used in referenced files
//                         deps.retain(|x| x != &name);
//                     }
//                 }
//                 parser::DartFile::Part(value) => {
//                     // part files
//                     let mut file = file_path.clone();
//                     file.set_file_name(value);
//                     extracted_data.referenced_files.insert(file);
//                 }
//                 parser::DartFile::Export(path) => {
//                     let file = path.replace("%20", " ");
//                     let file = Path::new(&file);
//                     let file = file_path.parent().unwrap().join(file);
//                     if !extracted_data
//                         .referenced_files
//                         .contains(&file.to_path_buf())
//                     {
//                         extracted_data
//                             .referenced_files
//                             .insert(file.parse_dot().unwrap().to_path_buf());
//                         extract_data(
//                             &file.parse_dot().unwrap().to_path_buf(),
//                             package_name,
//                             extracted_data,
//                             deps,
//                             assets,
//                             args,
//                         )?;
//                     }
//                 }
//             }
//         }
//     }

//     let mut remove = false;
//     let mut referenced_asset_files = HashSet::with_capacity(10);
//     for asset in assets.iter() {
//         if contents.contains(asset.borrow_file_name()) {
//             remove = true;
//             referenced_asset_files.insert(asset.borrow_path().clone());
//         }
//     }
//     // Remove referenced assets from the set to speed up future checks
//     if remove {
//         assets.retain(|asset| !referenced_asset_files.contains(asset.borrow_path()));
//     }

//     remove = false;
//     let mut used_deps = HashSet::with_capacity(10);
//     for dep in deps.iter() {
//         if contents.contains(dep) {
//             remove = true;
//             used_deps.insert(dep.clone());
//         }
//     }

//     // Remove used deps from the set to speed up future checks
//     if remove {
//         deps.retain(|dep| !used_deps.contains(dep));
//     }

//     if args.labels {
//         let s = all_localisation(&contents);
//         if let Ok((_, keys)) = s {
//             for key in keys {
//                 extracted_data.labels_referenced.insert(key.to_owned());
//             }
//         }
//     }

//     if args.loc
//         && let Ok((_, r)) = locator::locator(&contents)
//     {
//         for reg in r {
//             match reg {
//                 locator::Locator::Register(s) => {
//                     extracted_data.locators.entry(s).or_insert(false);
//                 }
//                 locator::Locator::Get(s) => {
//                     extracted_data.locators.insert(s, true);
//                 }
//                 _ => {}
//             }
//         }
//     }

//     Ok(())
// }
