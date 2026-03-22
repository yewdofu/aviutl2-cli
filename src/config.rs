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
    pub name: String,
    #[serde(rename = "type")]
    pub catalog_type: CatalogType,
    pub author: String,
    pub original_author: Option<String>,
    pub niconi_commons_id: Option<String>,
    pub summary: String,
    pub homepage: String,
    pub dependencies: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub description: CatalogDescription,
    pub license: CatalogLicense,
    pub download_source: CatalogDownloadSource,
    pub install_steps: Option<Vec<CatalogAction>>,
    pub uninstall_steps: Option<Vec<CatalogAction>>,
}

#[derive(Clone, PartialEq)]
pub enum CatalogType {
    Output,
    Input,
    Filter,
    Common,
    Modification,
    Script,
    Language,
    Other,
    Custom(String),
}

impl<'de> Deserialize<'de> for CatalogType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let catalog_type = match s.as_str() {
            "output" => CatalogType::Output,
            "input" => CatalogType::Input,
            "filter" => CatalogType::Filter,
            "common" => CatalogType::Common,
            "modification" => CatalogType::Modification,
            "script" => CatalogType::Script,
            "language" => CatalogType::Language,
            "other" => CatalogType::Other,
            custom => CatalogType::Custom(custom.to_string()),
        };
        Ok(catalog_type)
    }
}
impl Serialize for CatalogType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            CatalogType::Output => "output",
            CatalogType::Input => "input",
            CatalogType::Filter => "filter",
            CatalogType::Common => "common",
            CatalogType::Modification => "modification",
            CatalogType::Script => "script",
            CatalogType::Language => "language",
            CatalogType::Other => "other",
            CatalogType::Custom(custom) => custom.as_str(),
        };
        serializer.serialize_str(s)
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CatalogLicenseText {
    Inline { content: String },
    File { path: String },
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum CatalogDescription {
    Plain(String),
    Url(CatalogDescriptionUrl),
    Inline(CatalogDescriptionInline),
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct CatalogDescriptionUrl {
    #[serde(rename = "type")]
    pub description_type: CatalogDescriptionType,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct CatalogDescriptionInline {
    #[serde(rename = "type")]
    pub description_type: CatalogDescriptionType,
    pub content: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogDescriptionType {
    Url,
    Inline,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum CatalogLicense {
    Template(TemplateCatalogLicense),
    Custom(CustomCatalogLicense),
    Cc0(CC0License),
    Other(OtherCatalogLicense),
    Unknown(UnknownCatalogLicense),
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct TemplateCatalogLicense {
    #[serde(rename = "type")]
    pub license_type: TemplateCatalogLicenseType,
    pub template: serde_constant::ConstBool<true>,
    pub author: String,
    pub year: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct CustomCatalogLicense {
    #[serde(rename = "type")]
    pub license_type: TemplateCatalogLicenseType,
    pub template: serde_constant::ConstBool<false>,
    pub text: CatalogLicenseText,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct CC0License {
    #[serde(rename = "type")]
    pub license_type: CC0LicenseType,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct OtherCatalogLicense {
    #[serde(rename = "type")]
    pub license_type: OtherCatalogLicenseType,
    pub name: Option<String>,
    pub text: CatalogLicenseText,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct UnknownCatalogLicense {
    #[serde(rename = "type")]
    pub license_type: UnknownCatalogLicenseType,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum TemplateCatalogLicenseType {
    #[serde(rename = "MIT")]
    Mit,
    #[serde(rename = "Apache-2.0")]
    Apache20,
    #[serde(rename = "BSD-2-Clause")]
    Bsd2Clause,
    #[serde(rename = "BSD-3-Clause")]
    Bsd3Clause,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum CC0LicenseType {
    #[serde(rename = "CC0-1.0")]
    Cc0,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum OtherCatalogLicenseType {
    #[serde(rename = "other")]
    Other,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub enum UnknownCatalogLicenseType {
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum CatalogDownloadSource {
    #[serde(rename = "direct")]
    Direct { url: String },
    #[serde(rename = "booth")]
    Booth { url: String },
    #[serde(rename = "github")]
    Github {
        owner: String,
        repo: String,
        pattern: Option<String>,
    },
    #[serde(rename = "google_drive")]
    GoogleDrive { id: String },
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(tag = "action")]
pub enum CatalogAction {
    #[serde(rename = "download")]
    Download,
    #[serde(rename = "extract")]
    Extract,
    #[serde(rename = "copy")]
    Copy { from: String, to: String },
    #[serde(rename = "delete")]
    Delete { path: String },
    #[serde(rename = "run")]
    Run {
        path: String,
        args: Vec<String>,
        elevate: Option<bool>,
    },
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
        let parent = abs_path
            .parent()
            .with_context(|| format!("親ディレクトリの取得に失敗しました: {}", abs_path.display()))?;
        std::env::set_current_dir(parent).with_context(|| {
            format!(
                "カレントディレクトリの変更に失敗しました: {}",
                parent.display()
            )
        })?;
        fs::read_to_string(&abs_path)
            .with_context(|| format!("設定ファイルの読み込みに失敗しました: {}", abs_path.display()))?
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
        let merged = toml::to_string(&base).with_context(|| "設定ファイルのシリアライズに失敗しました")?;
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
