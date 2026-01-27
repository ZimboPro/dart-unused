use std::{collections::HashMap, path::PathBuf};

use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn get_package_details() -> anyhow::Result<PubspecSchema> {
    let pubspec = std::fs::read_to_string("pubspec.yaml").expect("Failed to read pubspec.yaml");

    serde_yaml2::from_str(&pubspec).map_err(|e| e.into())
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PubspecSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_tracker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_overrides: Option<HashMap<String, Dependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executables: Option<HashMap<String, Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<HashMap<String, Option<Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_to: Option<PublishTo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funding: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub false_secrets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshots: Option<Vec<Screenshot>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_advisories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Environment>,
    #[serde(default)]
    pub flutter: Flutter,
    #[serde(default)]
    pub flutter_intl: FlutterIntl,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Environment {
    pub sdk: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flutter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version constraint: "^1.0.0" or ">=1.0.0 <2.0.0"
    Version(String),
    /// Path dependency
    Path {
        path: PathBuf,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
    /// SDK dependency (flutter, dart)
    SDK {
        sdk: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
    /// Git dependency with URL and optional ref/tag_pattern
    Git {
        git: GitDependency,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
    /// Hosted dependency (custom package server)
    Hosted {
        hosted: HostedDependency,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum GitDependency {
    /// Simple git URL: git: https://github.com/user/repo.git
    Url(String),
    /// Full git specification
    Full {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "ref")]
        git_ref: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tag_pattern: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HostedDependency {
    /// Simple hosted URL: hosted: https://custom-pub-server.com
    Url(String),
    /// Full hosted specification (legacy format)
    Full { name: String, url: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublishTo {
    /// Disable publishing: publish_to: none
    None,
    /// Custom server URL
    #[serde(untagged)]
    Url(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Screenshot {
    pub description: String,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default, Clone)]
pub struct Flutter {
    #[serde(default)]
    #[serde(alias = "uses-material-design")]
    pub uses_material_design: bool,
    #[serde(default)]
    pub generate: bool,
    #[serde(default)]
    pub assets: Vec<AssetElement>,
    #[serde(default)]
    pub fonts: Vec<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licenses: Option<Vec<PathBuf>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shaders: Option<Vec<PathBuf>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<FlutterPlugin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "default-flavor")]
    pub default_flavor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "deferred-components")]
    pub deferred_components: Option<Vec<DeferredComponent>>,
}

impl Flutter {
    pub fn get_asset_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for asset in &self.assets {
            match asset {
                AssetElement::AssetClass(ac) => {
                    paths.push(ac.path.clone());
                }
                AssetElement::Path(p) => {
                    paths.push(p.clone());
                }
            }
        }
        paths
    }

    pub fn get_assets(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for asset in &self.assets {
            match asset {
                AssetElement::AssetClass(ac) => {
                    if ac.path.is_file() {
                        paths.push(ac.path.clone());
                    } else if ac.path.is_dir() {
                        let pattern = format!("{}/*", ac.path.to_str().unwrap());
                        let files = glob(&pattern)
                            .expect("Failed to read glob pattern")
                            .flatten()
                            .collect::<Vec<PathBuf>>();
                        for file in files {
                            if file.is_file() {
                                paths.push(file);
                            }
                        }
                    }
                }
                AssetElement::Path(p) => {
                    if p.is_file() {
                        paths.push(p.clone());
                    } else if p.is_dir() {
                        let pattern = format!("{}/*", p.to_str().unwrap());
                        let files = glob(&pattern)
                            .expect("Failed to read glob pattern")
                            .flatten()
                            .collect::<Vec<PathBuf>>();
                        for file in files {
                            if file.is_file() {
                                paths.push(file);
                            }
                        }
                    }
                }
            }
        }
        paths
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AssetClass {
    path: PathBuf,
    flavors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum AssetElement {
    AssetClass(AssetClass),

    Path(PathBuf),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct FlutterPlugin {
    pub platforms: HashMap<String, PlatformConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implements: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PlatformConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pluginClass")]
    pub plugin_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dartPluginClass")]
    pub dart_plugin_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ffiPlugin")]
    pub ffi_plugin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "default_package")]
    pub default_package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sharedDarwinSource")]
    pub shared_darwin_source: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct DeferredComponent {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libraries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<PathBuf>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FlutterIntl {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_class_name")]
    pub class_name: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default = "default_arb_dir")]
    pub arb_dir: PathBuf,
    #[serde(default = "default_main_locale")]
    pub main_locale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localize_function_name: Option<String>,
}

fn default_class_name() -> String {
    "AppLocalizations".to_string()
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("lib/generated")
}

fn default_arb_dir() -> PathBuf {
    PathBuf::from("lib/l10n")
}

fn default_main_locale() -> String {
    "en".to_string()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Font {
    pub family: String,
    pub fonts: Vec<FontFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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
    use super::*;
    use serde::de::value::Error;

    #[test]
    fn test_simple_dependencies() {
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
                    version: None,
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
                    version: None,
                },
            ),
        ]
        .into_iter()
        .collect();

        let result: Result<HashMap<String, Dependency>, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(dependencies));
    }

    #[test]
    fn test_git_dependencies() {
        let input = r#"
kittens:
    git: https://github.com/munificent/kittens.git
other_package:
    git:
        url: git@github.com:user/repo.git
        ref: some-branch
        path: packages/my_package
        "#;
        let dependencies: HashMap<String, Dependency> = vec![
            (
                "kittens".to_string(),
                Dependency::Git {
                    git: GitDependency::Url(
                        "https://github.com/munificent/kittens.git".to_string(),
                    ),
                    version: None,
                },
            ),
            (
                "other_package".to_string(),
                Dependency::Git {
                    git: GitDependency::Full {
                        url: "git@github.com:user/repo.git".to_string(),
                        git_ref: Some("some-branch".to_string()),
                        path: Some(PathBuf::from("packages/my_package")),
                        tag_pattern: None,
                    },
                    version: None,
                },
            ),
        ]
        .into_iter()
        .collect();

        let result: Result<HashMap<String, Dependency>, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(dependencies));
    }

    #[test]
    fn test_hosted_dependencies() {
        let input = r#"
transmogrify:
    hosted: https://custom-server.com
    version: ^1.4.0
legacy_package:
    hosted:
        name: legacy_package
        url: https://old-server.com
    version: ^0.1.0
        "#;
        let dependencies: HashMap<String, Dependency> = vec![
            (
                "transmogrify".to_string(),
                Dependency::Hosted {
                    hosted: HostedDependency::Url("https://custom-server.com".to_string()),
                    version: Some("^1.4.0".to_string()),
                },
            ),
            (
                "legacy_package".to_string(),
                Dependency::Hosted {
                    hosted: HostedDependency::Full {
                        name: "legacy_package".to_string(),
                        url: "https://old-server.com".to_string(),
                    },
                    version: Some("^0.1.0".to_string()),
                },
            ),
        ]
        .into_iter()
        .collect();

        let result: Result<HashMap<String, Dependency>, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(dependencies));
    }

    #[test]
    fn test_comprehensive_pubspec() {
        let input = r#"name: flutter_app
version: 1.0.0+1
description: A comprehensive Flutter project showcasing all pubspec features.
homepage: https://example.com
repository: https://github.com/user/flutter_app
issue_tracker: https://github.com/user/flutter_app/issues
documentation: https://docs.example.com
publish_to: none

environment:
  sdk: ^3.2.0
  flutter: '>=3.22.0'

dependencies:
  flutter:
    sdk: flutter
  cupertino_icons: ^1.0.8
  http: ^1.0.0
  local_package:
    path: ../local_package
  git_package:
    git:
      url: https://github.com/user/git_package.git
      ref: main

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^6.0.0
  mockito: ^5.0.0

dependency_overrides:
  transmogrify:
    path: ../transmogrify_patch

executables:
  my_script: bin_script
  default_script:

platforms:
  android:
  ios:
  linux:
  macos:
  web:
  windows:

funding:
  - https://github.com/sponsors/user
  - https://www.buymeacoffee.com/user

false_secrets:
  - /lib/src/hardcoded_key.dart
  - /test/certificates/*.pem

screenshots:
  - description: 'Main app interface showing the dashboard'
    path: screenshots/dashboard.png
  - description: 'Settings page with various options'
    path: screenshots/settings.png

topics:
  - flutter
  - mobile
  - cross-platform
  - ui

ignored_advisories:
  - GHSA-4rgh-jx4f-qfcq

flutter:
  uses-material-design: true
  generate: true

  config:
    enable-swift-package-manager: true
    cli-animations: false

  default-flavor: production

  assets:
    - assets/images/
    - assets/icons/app_icon.png
    - path: assets/flavor_specific/
      flavors:
        - staging
        - production

  fonts:
    - family: CustomFont
      fonts:
        - asset: fonts/CustomFont-Regular.ttf
        - asset: fonts/CustomFont-Bold.ttf
          weight: 700
          style: italic

  licenses:
    - assets/license.txt
    - assets/third_party_licenses.txt

  shaders:
    - assets/shaders/fragment_shader.frag

  plugin:
    platforms:
      android:
        package: com.example.flutter_app
        pluginClass: FlutterAppPlugin
        dartPluginClass: FlutterAppDartPlugin
        ffiPlugin: false
      ios:
        pluginClass: FlutterAppPlugin
        dartPluginClass: FlutterAppDartPlugin
        sharedDarwinSource: true
    implements:
      - platform_interface

  deferred-components:
    - name: feature_a
      libraries:
        - package:flutter_app/feature_a.dart
      assets:
        - assets/feature_a/
    - name: feature_b
      libraries:
        - package:flutter_app/feature_b.dart
        "#;

        let result: Result<PubspecSchema, Error> = serde_yaml2::from_str(input);
        assert!(
            result.is_ok(),
            "Failed to parse comprehensive pubspec: {:?}",
            result.err()
        );

        let pubspec = result.unwrap();
        assert_eq!(pubspec.name, "flutter_app");
        assert_eq!(pubspec.version, Some("1.0.0+1".to_string()));
        assert_eq!(
            pubspec.description,
            Some("A comprehensive Flutter project showcasing all pubspec features.".to_string())
        );
        assert_eq!(pubspec.homepage, Some("https://example.com".to_string()));
        assert_eq!(
            pubspec.repository,
            Some("https://github.com/user/flutter_app".to_string())
        );
        assert_eq!(
            pubspec.issue_tracker,
            Some("https://github.com/user/flutter_app/issues".to_string())
        );
        assert_eq!(
            pubspec.documentation,
            Some("https://docs.example.com".to_string())
        );
        assert_eq!(pubspec.publish_to, Some(PublishTo::None));

        // Test environment
        let env = pubspec.environment.unwrap();
        assert_eq!(env.sdk, "^3.2.0");
        assert_eq!(env.flutter, Some(">=3.22.0".to_string()));

        // Test dependencies
        assert!(pubspec.dependencies.contains_key("flutter"));
        assert!(pubspec.dependencies.contains_key("cupertino_icons"));
        assert!(pubspec.dependencies.contains_key("local_package"));
        assert!(pubspec.dependencies.contains_key("git_package"));

        // Test dev_dependencies
        assert!(pubspec.dev_dependencies.contains_key("flutter_test"));
        assert!(pubspec.dev_dependencies.contains_key("flutter_lints"));

        // Test dependency_overrides
        let overrides = pubspec.dependency_overrides.unwrap();
        assert!(overrides.contains_key("transmogrify"));

        // Test executables
        let executables = pubspec.executables.unwrap();
        assert_eq!(
            executables.get("my_script"),
            Some(&Some("bin_script".to_string()))
        );
        assert_eq!(executables.get("default_script"), Some(&None));

        // Test platforms
        let platforms = pubspec.platforms.unwrap();
        assert!(platforms.contains_key("android"));
        assert!(platforms.contains_key("ios"));

        // Test funding
        let funding = pubspec.funding.unwrap();
        assert_eq!(funding.len(), 2);
        assert!(funding.contains(&"https://github.com/sponsors/user".to_string()));

        // Test false_secrets
        let false_secrets = pubspec.false_secrets.unwrap();
        assert_eq!(false_secrets.len(), 2);
        assert!(false_secrets.contains(&"/lib/src/hardcoded_key.dart".to_string()));

        // Test screenshots
        let screenshots = pubspec.screenshots.unwrap();
        assert_eq!(screenshots.len(), 2);
        assert_eq!(
            screenshots[0].description,
            "Main app interface showing the dashboard"
        );
        assert_eq!(
            screenshots[0].path,
            PathBuf::from("screenshots/dashboard.png")
        );

        // Test topics
        let topics = pubspec.topics.unwrap();
        assert_eq!(topics.len(), 4);
        assert!(topics.contains(&"flutter".to_string()));
        assert!(topics.contains(&"mobile".to_string()));

        // Test ignored_advisories
        let advisories = pubspec.ignored_advisories.unwrap();
        assert_eq!(advisories.len(), 1);
        assert!(advisories.contains(&"GHSA-4rgh-jx4f-qfcq".to_string()));

        // Test flutter section
        let flutter = pubspec.flutter.clone();
        assert!(flutter.uses_material_design);
        assert!(flutter.generate);

        // Test flutter config
        let config = flutter.config.clone().unwrap();
        assert_eq!(config.get("enable-swift-package-manager"), Some(&true));
        assert_eq!(config.get("cli-animations"), Some(&false));

        // Test flutter assets
        assert_eq!(flutter.assets.len(), 3);

        // Test flutter fonts
        assert_eq!(flutter.fonts.len(), 1);
        assert_eq!(flutter.fonts[0].family, "CustomFont");
        assert_eq!(flutter.fonts[0].fonts.len(), 2);

        // Test flutter plugin
        let plugin = flutter.plugin.unwrap();
        assert!(plugin.platforms.contains_key("android"));
        assert!(plugin.platforms.contains_key("ios"));
        let implements = plugin.implements.unwrap();
        assert!(implements.contains(&"platform_interface".to_string()));

        // Test deferred components
        let deferred = flutter.deferred_components.unwrap();
        assert_eq!(deferred.len(), 2);
        assert_eq!(deferred[0].name, "feature_a");
        assert_eq!(deferred[1].name, "feature_b");
    }

    #[test]
    fn test_minimal_pubspec() {
        let input = r#"name: minimal_app"#;
        let result: Result<PubspecSchema, Error> = serde_yaml2::from_str(input);
        assert!(result.is_ok());

        let pubspec = result.unwrap();
        assert_eq!(pubspec.name, "minimal_app");
        assert_eq!(pubspec.version, None);
        assert_eq!(pubspec.description, None);
        assert!(pubspec.dependencies.is_empty());
        assert!(pubspec.dev_dependencies.is_empty());
    }

    // #[test]
    //    fn test_flutter_assets_flavors() {
    //        let input = r#"
    //flutter:
    //  assets:
    //    - assets/common/
    //    - path: assets/staging/
    //      flavors:
    //        - staging
    //    - path: assets/production/
    //      flavors:
    //        - production
    //        "#;
    //
    //        let result: Result<Flutter, Error> = serde_yaml2::from_str(input);
    //        assert!(result.is_ok());
    //
    //        let flutter = result.unwrap();
    //        assert_eq!(flutter.assets.len(), 3);
    //
    //        match &flutter.assets[0] {
    //            AssetElement::Path(path) => assert_eq!(*path, PathBuf::from("assets/common/")),
    //            _ => panic!("Expected simple path asset"),
    //        }
    //
    //        match &flutter.assets[1] {
    //            AssetElement::AssetClass(AssetClass { path, flavors }) => {
    //                assert_eq!(*path, PathBuf::from("assets/staging/"));
    //                assert_eq!(flavors, &vec!["staging".to_string()]);
    //            }
    //            _ => panic!("Expected flavored asset"),
    //        }
    //    }
    //
    #[test]
    fn test_dependency_version_constraints() {
        let input = r#"
http: ^1.0.0
path: '>=1.8.0 <2.0.0'
exact_version: 1.2.3
any_version: any
        "#;

        let dependencies: HashMap<String, Dependency> = vec![
            (
                "http".to_string(),
                Dependency::Version("^1.0.0".to_string()),
            ),
            (
                "path".to_string(),
                Dependency::Version(">=1.8.0 <2.0.0".to_string()),
            ),
            (
                "exact_version".to_string(),
                Dependency::Version("1.2.3".to_string()),
            ),
            (
                "any_version".to_string(),
                Dependency::Version("any".to_string()),
            ),
        ]
        .into_iter()
        .collect();

        let result: Result<HashMap<String, Dependency>, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(dependencies));
    }

    #[test]
    fn test_publish_to_variants() {
        // Test none
        let input_none = "none";
        let result: Result<PublishTo, Error> = serde_yaml2::from_str(input_none);
        assert_eq!(result, Ok(PublishTo::None));

        // Test custom URL
        let input_url = "https://custom-pub-server.com";
        let result: Result<PublishTo, Error> = serde_yaml2::from_str(input_url);
        assert_eq!(
            result,
            Ok(PublishTo::Url("https://custom-pub-server.com".to_string()))
        );
    }

    #[test]
    fn test_environment_constraints() {
        let input = r#"
sdk: ^3.2.0
flutter: '>=3.22.0'
        "#;

        let expected = Environment {
            sdk: "^3.2.0".to_string(),
            flutter: Some(">=3.22.0".to_string()),
        };

        let result: Result<Environment, Error> = serde_yaml2::from_str(input);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_font_configurations() {
        let input = r#"
- family: Roboto
  fonts:
    - asset: fonts/Roboto-Regular.ttf
    - asset: fonts/Roboto-Bold.ttf
      weight: 700
    - asset: fonts/Roboto-Italic.ttf
      style: italic
    - asset: fonts/Roboto-BoldItalic.ttf
      weight: 700
      style: italic
        "#;

        let result: Result<Vec<Font>, Error> = serde_yaml2::from_str(input);
        assert!(result.is_ok());

        let fonts = result.unwrap();
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].family, "Roboto");
        assert_eq!(fonts[0].fonts.len(), 4);

        assert_eq!(
            fonts[0].fonts[0].asset,
            PathBuf::from("fonts/Roboto-Regular.ttf")
        );
        assert_eq!(fonts[0].fonts[0].weight, None);
        assert_eq!(fonts[0].fonts[0].style, None);

        assert_eq!(
            fonts[0].fonts[1].asset,
            PathBuf::from("fonts/Roboto-Bold.ttf")
        );
        assert_eq!(fonts[0].fonts[1].weight, Some(700));
        assert_eq!(fonts[0].fonts[1].style, None);

        assert_eq!(
            fonts[0].fonts[2].asset,
            PathBuf::from("fonts/Roboto-Italic.ttf")
        );
        assert_eq!(fonts[0].fonts[2].weight, None);
        assert_eq!(fonts[0].fonts[2].style, Some("italic".to_string()));
    }
}
