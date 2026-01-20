use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use clap::Parser;
use glob::glob;
use path_dedot::ParseDot;

pub mod assets;
pub mod cli;
pub mod config;
pub mod parser;
pub mod pubspec;
pub mod util;

use crate::assets::{OsStringWithStr, get_assets, get_registered_assets};

pub fn get_unreferenced_files() -> anyhow::Result<()> {
    let args = cli::CLI::parse();
    let config: config::Config = if let Ok(s) = std::fs::read_to_string("unused.config.yaml") {
        serde_yaml2::from_str(&s).unwrap()
    } else {
        Default::default()
    };

    util::set_current_dir(&args.path)?;
    let pubspec = pubspec::get_package_details()?;
    let mut assets = get_assets(args.assets, &pubspec.flutter.assets, &config.assets.ignore)?;
    let mut deps: Vec<String> = if args.deps {
        pubspec.dependencies.keys().cloned().collect()
    } else {
        Vec::new()
    };
    let mut referenced_files: HashSet<PathBuf> = HashSet::with_capacity(10_000);
    // TODO allow to set entry point
    let main = PathBuf::from("lib/main.dart");
    referenced_files.insert(main.clone());
    extract_data(
        &main,
        &pubspec.name,
        &mut referenced_files,
        &mut deps,
        &mut assets,
    )?;

    let dart = glob("lib/**/*.dart").expect("Failed to read glob pattern");
    let mut dart: Vec<PathBuf> = dart.flatten().collect();
    dart.retain(|path| !referenced_files.contains(path));
    if !assets.is_empty() {
        let assets: HashSet<PathBuf> = assets
            .into_iter()
            .map(|x| x.borrow_os().clone().into())
            .collect();
        let mut all_assets = get_registered_assets(&pubspec.flutter.assets)?;
        all_assets.retain(|x| assets.contains(x));

        for asset in all_assets.iter().enumerate() {
            log::error!("{}. Unreferenced asset: {:?}", asset.0 + 1, asset.1);
        }
        log::info!("");
    }
    if args.deps {
        for dep in deps.iter().enumerate() {
            log::error!("{}. Unused dependencies: {:?}", dep.0 + 1, dep.1);
        }
        log::info!("");
    }

    for file in dart.iter().enumerate() {
        log::error!("{} Unreferenced file: {:?}", file.0 + 1, file.1);
    }
    Ok(())
}

fn extract_data(
    file_path: &std::path::PathBuf,
    package_name: &str,
    referenced_files: &mut HashSet<PathBuf>,
    deps: &mut Vec<String>,
    assets: &mut Vec<OsStringWithStr>,
) -> anyhow::Result<()> {
    let contents = std::fs::read_to_string(&file_path).expect("Failed to read file");
    for line in contents.lines() {
        if let Ok((_, dart)) = parser::dart_file(line) {
            match dart {
                parser::DartFile::Import(path) => {
                    // relative path imports
                    let file = path.replace("%20", " ");
                    let file = Path::new(&file);
                    let file = file_path.parent().unwrap().join(file);
                    if !referenced_files.contains(&file.to_path_buf()) {
                        referenced_files.insert(file.parse_dot().unwrap().to_path_buf());
                        extract_data(
                            &file.parse_dot().unwrap().to_path_buf(),
                            package_name,
                            referenced_files,
                            deps,
                            assets,
                        )?;
                    }
                }
                parser::DartFile::Package(name, mut path) => {
                    // package imports
                    if name == package_name {
                        path.insert_str(0, "lib");
                        let path = path.replace("%20", " ");
                        let file = Path::new(&path);
                        if !referenced_files.contains(&file.to_path_buf()) {
                            referenced_files.insert(file.to_path_buf());
                            extract_data(
                                &file.to_path_buf(),
                                package_name,
                                referenced_files,
                                deps,
                                assets,
                            )?;
                        }
                    } else {
                        // referenced_packages.push(DartFile::Package(name, path));
                        // Remove deps used in referenced files
                        deps.retain(|x| x != &name);
                    }
                }
                parser::DartFile::Part(value) => {
                    // part files
                    let mut file = file_path.clone();
                    file.set_file_name(value);
                    referenced_files.insert(file);
                }
                parser::DartFile::Export(path) => {
                    let file = path.replace("%20", " ");
                    let file = Path::new(&file);
                    let file = file_path.parent().unwrap().join(file);
                    if !referenced_files.contains(&file.to_path_buf()) {
                        referenced_files.insert(file.parse_dot().unwrap().to_path_buf());
                        extract_data(
                            &file.parse_dot().unwrap().to_path_buf(),
                            package_name,
                            referenced_files,
                            deps,
                            assets,
                        )?;
                    }
                }
            }
        }
    }

    let mut remove = false;
    let mut referenced_asset_files = HashSet::with_capacity(10);
    for asset in assets.iter() {
        if contents.contains(asset.borrow_s()) {
            remove = true;
            referenced_asset_files.insert(asset.borrow_os().clone());
        }
    }
    // Remove referenced assets from the set to speed up future checks
    if remove {
        assets.retain(|asset| !referenced_asset_files.contains(asset.borrow_os()));
    }

    remove = false;
    let mut used_deps = HashSet::with_capacity(10);
    for dep in deps.iter() {
        if contents.contains(dep) {
            remove = true;
            used_deps.insert(dep.clone());
        }
    }

    // Remove used deps from the set to speed up future checks
    if remove {
        deps.retain(|dep| !used_deps.contains(dep));
    }
    Ok(())
}
