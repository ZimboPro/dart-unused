use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

pub fn get_package_details() -> anyhow::Result<PubspecSchema> {
    let pubspec = std::fs::read_to_string("pubspec.yaml").expect("Failed to read pubspec.yaml");

    serde_yaml2::from_str(&pubspec).map_err(|e| e.into())
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PubspecSchema {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_overrides: Option<HashMap<String, Dependency>>,
    #[serde(default)]
    pub flutter: Flutter,
    pub environment: Environment,
    #[serde(default)]
    pub flutter_intl: FlutterIntl,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Environment {
    pub sdk: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Dependency {
    Version(String),
    Path { path: PathBuf },
    SDK { sdk: String },
    Hosted { hosted: String, version: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Flutter {
    #[serde(default)]
    #[serde(alias = "uses-material-design")]
    pub uses_material_design: bool,
    #[serde(default)]
    pub assets: Vec<PathBuf>,
    #[serde(default)]
    pub fonts: Vec<Font>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FlutterIntl {
    pub enabled: bool,
    pub class_name: String,
    pub output_dir: PathBuf,
    pub arb_dir: PathBuf,
    pub main_locale: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Font {
    pub family: String,
    pub fonts: Vec<FontFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FontFile {
    pub asset: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub weight: Option<u16>,
}

#[cfg(test)]
mod tests {
    use serde::de::value::Error;

    use super::*;

    #[test]
    fn test_pubspec_dependency() {
        let input = r#"
flutter:
    sdk: flutter
cupertino_icons: ^0.1.3
datadog_flutter_plugin: 2.1.1
sitemap_annotations:
    path: packages/sitemap_annotations
        "#;
        let dependencies: HashMap<String, Dependency> = vec![
            (
                "flutter".to_string(),
                Dependency::SDK {
                    sdk: "flutter".to_string(),
                },
            ),
            (
                "cupertino_icons".to_string(),
                Dependency::Version("^0.1.3".to_string()),
            ),
            (
                "datadog_flutter_plugin".to_string(),
                Dependency::Version("2.1.1".to_string()),
            ),
            (
                "sitemap_annotations".to_string(),
                Dependency::Path {
                    path: PathBuf::from("packages/sitemap_annotations"),
                },
            ),
        ]
        .into_iter()
        .collect();

        let result: Result<HashMap<String, Dependency>, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(dependencies));
    }

    #[test]
    fn test_pubspec() {
        let input = r#"name: flutter_app
version: 1.0.0
description: A new Flutter project.
dependencies:
    flutter:
        sdk: flutter
    cupertino_icons: ^0.1.3
    datadog_flutter_plugin: 2.1.1
    sitemap_annotations:
        path: packages/sitemap_annotations
dev_dependencies:
    flutter_test:
        sdk: flutter
environment:
    sdk: ">=2.12.0 <3.0.0"
        "#;
        let expected = PubspecSchema {
            name: "flutter_app".to_string(),
            version: "1.0.0".to_string(),
            description: "A new Flutter project.".to_string(),
            homepage: None,
            repository: None,
            dependencies: vec![
                (
                    "flutter".to_string(),
                    Dependency::SDK {
                        sdk: "flutter".to_string(),
                    },
                ),
                (
                    "cupertino_icons".to_string(),
                    Dependency::Version("^0.1.3".to_string()),
                ),
                (
                    "datadog_flutter_plugin".to_string(),
                    Dependency::Version("2.1.1".to_string()),
                ),
                (
                    "sitemap_annotations".to_string(),
                    Dependency::Path {
                        path: PathBuf::from("packages/sitemap_annotations"),
                    },
                ),
            ]
            .into_iter()
            .collect(),

            dev_dependencies: vec![(
                "flutter_test".to_string(),
                Dependency::SDK {
                    sdk: "flutter".to_string(),
                },
            )]
            .into_iter()
            .collect(),
            // ),
            dependency_overrides: None,
            flutter: Default::default(),
            environment: Environment {
                sdk: ">=2.12.0 <3.0.0".to_string(),
            },
            flutter_intl: Default::default(),
        };

        let result: Result<PubspecSchema, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(expected));
    }
}
