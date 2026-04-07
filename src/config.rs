use anyhow::{Context, Result, bail};
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub project: Project,
    #[serde(default)]
    pub artifacts: HashMap<String, Artifact>,
    #[serde(default)]
    pub build_group: HashMap<String, BuildCommand>,
    pub development: Option<Development>,
    #[serde(default)]
    pub preview: Preview,
    #[serde(default)]
    pub release: Release,
    pub catalog: Option<Catalog>,
}

#[derive(Deserialize)]
pub struct Project {
    pub id: String,
    pub name: Option<String>,
    pub version: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Artifact {
    pub enabled: Option<bool>,
    pub source: Option<String>,
    pub destination: String,
    pub build: Option<BuildCommand>,
    pub placement_method: Option<PlacementMethod>,
    pub profiles: Option<HashMap<String, ArtifactProfile>>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct ArtifactProfile {
    pub enabled: Option<bool>,
    pub source: Option<String>,
    pub build: Option<BuildCommand>,
}

#[derive(Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlacementMethod {
    Symlink,
    Copy,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum BuildCommand {
    Single(String),
    Multiple(Vec<String>),
    Group(BuildGroupRef),
}

impl std::default::Default for BuildCommand {
    fn default() -> Self {
        BuildCommand::Multiple(vec![])
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct BuildGroupRef {
    pub group: String,
}

#[derive(Deserialize)]
pub struct Development {
    pub aviutl2_version: String,
    #[serde(default = "Development::default_install_dir")]
    pub install_dir: String,
    #[serde(default = "Development::default_profile")]
    pub profile: String,
    #[serde(default)]
    pub prebuild: BuildCommand,
    #[serde(default)]
    pub postbuild: BuildCommand,
    #[serde(default)]
    pub catalog_dependencies: Vec<CatalogDependency>,
}

impl Development {
    pub fn default_install_dir() -> String {
        ".aviutl2-cli/development".to_string()
    }

    pub fn default_profile() -> String {
        "debug".to_string()
    }
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Preview {
    pub aviutl2_version: Option<String>,
    #[serde(default = "Preview::default_install_dir")]
    pub install_dir: String,
    #[serde(default = "Preview::default_profile")]
    pub profile: String,
    pub include: Option<Vec<String>>,
    #[serde(default)]
    pub prebuild: BuildCommand,
    #[serde(default)]
    pub postbuild: BuildCommand,
    #[serde(default)]
    pub catalog_dependencies: Vec<CatalogDependency>,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            aviutl2_version: None,
            install_dir: Self::default_install_dir(),
            profile: Self::default_profile(),
            include: None,
            prebuild: BuildCommand::default(),
            postbuild: BuildCommand::default(),
            catalog_dependencies: Vec::new(),
        }
    }
}

impl Preview {
    pub fn default_install_dir() -> String {
        ".aviutl2-cli/preview".to_string()
    }

    pub fn default_profile() -> String {
        "release".to_string()
    }
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct Release {
    #[serde(default = "Release::default_output_dir")]
    pub output_dir: String,
    pub package_template: Option<String>,
    #[serde(default = "Release::default_package_id")]
    pub package_id: String,
    #[serde(default = "Release::default_package_name")]
    pub package_name: String,
    #[serde(default = "Release::default_package_information")]
    pub package_information: String,
    #[serde(default = "Release::default_zip_name")]
    pub zip_name: String,
    #[serde(default = "Release::default_profile")]
    pub profile: String,
    pub include: Option<Vec<String>>,
    #[serde(default)]
    pub prebuild: BuildCommand,
    #[serde(default)]
    pub postbuild: BuildCommand,
}

impl Default for Release {
    fn default() -> Self {
        Self {
            output_dir: Self::default_output_dir(),
            package_template: None,
            package_id: Self::default_package_id(),
            package_name: Self::default_package_name(),
            package_information: Self::default_package_information(),
            zip_name: Self::default_zip_name(),
            profile: Self::default_profile(),
            include: None,
            prebuild: BuildCommand::default(),
            postbuild: BuildCommand::default(),
        }
    }
}

impl Release {
    pub fn default_output_dir() -> String {
        "release".to_string()
    }

    pub fn default_package_id() -> String {
        "{id}".to_string()
    }

    pub fn default_package_name() -> String {
        "{name}".to_string()
    }

    pub fn default_package_information() -> String {
        "{name} v{version}".to_string()
    }

    pub fn default_zip_name() -> String {
        "{id}-v{version}".to_string()
    }

    pub fn default_profile() -> String {
        "release".to_string()
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct Catalog {
    pub id: String,
    pub description_path: Option<String>,
    pub license_path: Option<String>,
    pub download_repo: Option<CatalogDownloadRepo>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct CatalogDownloadRepo {
    pub owner: String,
    pub repo: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CatalogDependency {
    pub id: String,
}

impl<'de> Deserialize<'de> for CatalogDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum CatalogDependencyDef {
            Simple(String),
            Detailed { id: String },
        }

        let def = CatalogDependencyDef::deserialize(deserializer)?;
        let dependency = match def {
            CatalogDependencyDef::Simple(id) => CatalogDependency { id },
            CatalogDependencyDef::Detailed { id } => CatalogDependency { id },
        };
        Ok(dependency)
    }
}

pub struct ConfigLoadOpts {
    pub patch: Option<String>,
    pub override_path: Option<String>,
}

pub fn load_config(opts: &ConfigLoadOpts) -> Result<Config> {
    if opts.patch.is_some() && opts.override_path.is_some() {
        bail!("-c と -C は同時に指定できません");
    }

    let content = if let Some(override_path) = &opts.override_path {
        let path = PathBuf::from(override_path);
        let abs_path = path
            .canonicalize()
            .with_context(|| format!("ファイルが見つかりません: {}", path.display()))?;
        let parent = abs_path.parent().with_context(|| {
            format!("親ディレクトリの取得に失敗しました: {}", abs_path.display())
        })?;
        std::env::set_current_dir(parent).with_context(|| {
            format!(
                "カレントディレクトリの変更に失敗しました: {}",
                parent.display()
            )
        })?;
        fs::read_to_string(&abs_path).with_context(|| {
            format!(
                "設定ファイルの読み込みに失敗しました: {}",
                abs_path.display()
            )
        })?
    } else {
        let path = find_and_cd_to_project()?;
        fs::read_to_string(&path)
            .with_context(|| format!("設定ファイルの読み込みに失敗しました: {}", path.display()))?
    };

    if let Some(patch_path) = &opts.patch {
        let patch_path = PathBuf::from(patch_path);
        let patch_content = fs::read_to_string(&patch_path).with_context(|| {
            format!(
                "パッチファイルの読み込みに失敗しました: {}",
                patch_path.display()
            )
        })?;
        let mut base: toml::Value =
            toml::from_str(&content).with_context(|| "設定ファイルの解析に失敗しました")?;
        let patch: toml::Value =
            toml::from_str(&patch_content).with_context(|| "パッチファイルの解析に失敗しました")?;
        merge_toml(&mut base, patch);
        let merged =
            toml::to_string(&base).with_context(|| "設定ファイルのシリアライズに失敗しました")?;
        toml::from_str(&merged).with_context(|| "設定ファイルの解析に失敗しました")
    } else {
        toml::from_str(&content).with_context(|| "設定ファイルの解析に失敗しました")
    }
}

fn merge_toml(base: &mut toml::Value, patch: toml::Value) {
    match (&mut *base, patch) {
        (toml::Value::Table(base_table), toml::Value::Table(patch_table)) => {
            for (key, patch_val) in patch_table {
                match base_table.get_mut(&key) {
                    Some(base_val) => merge_toml(base_val, patch_val),
                    None => {
                        base_table.insert(key, patch_val);
                    }
                }
            }
        }
        (base, patch) => *base = patch,
    }
}

pub fn find_and_cd_to_project() -> Result<PathBuf> {
    static CANDIDATE_FILES: &[&str] = &["aviutl2.toml", ".aviutl2.toml", ".config/aviutl2.toml"];
    let mut current =
        std::env::current_dir().context("カレントディレクトリの取得に失敗しました")?;
    loop {
        for candidate in CANDIDATE_FILES {
            let candidate_path = current.join(candidate);
            if candidate_path.is_file() {
                std::env::set_current_dir(&current).with_context(|| {
                    format!(
                        "カレントディレクトリの変更に失敗しました: {}",
                        current.display()
                    )
                })?;
                return Ok(candidate_path);
            }
        }
        if !current.pop() {
            bail!("設定ファイルが見つかりませんでした");
        }
    }
}
