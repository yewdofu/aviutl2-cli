use serde::{Deserialize, Serialize};

/* ---------- primitives ---------- */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseType {
    Mit,
    Apache20,
    Bsd2Clause,
    Bsd3Clause,
    Cc010,
    Gpl20,
    Gpl30,
    Unlicense,
    Custom,
    Other(String),
}

impl<'de> Deserialize<'de> for LicenseType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "MIT" => LicenseType::Mit,
            "Apache-2.0" => LicenseType::Apache20,
            "BSD-2-Clause" => LicenseType::Bsd2Clause,
            "BSD-3-Clause" => LicenseType::Bsd3Clause,
            "CC0-1.0" => LicenseType::Cc010,
            "GPL-2.0" => LicenseType::Gpl20,
            "GPL-3.0" => LicenseType::Gpl30,
            "Unlicense" => LicenseType::Unlicense,
            "カスタムライセンス" => LicenseType::Custom,
            other => LicenseType::Other(other.to_string()),
        })
    }
}

impl Serialize for LicenseType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            LicenseType::Mit => "MIT",
            LicenseType::Apache20 => "Apache-2.0",
            LicenseType::Bsd2Clause => "BSD-2-Clause",
            LicenseType::Bsd3Clause => "BSD-3-Clause",
            LicenseType::Cc010 => "CC0-1.0",
            LicenseType::Gpl20 => "GPL-2.0",
            LicenseType::Gpl30 => "GPL-3.0",
            LicenseType::Unlicense => "Unlicense",
            LicenseType::Custom => "カスタムライセンス",
            LicenseType::Other(other) => other.as_str(),
        };
        serializer.serialize_str(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Copyright {
    pub years: String,
    pub holder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    #[serde(rename = "type")]
    pub license_type: LicenseType,

    #[serde(rename = "isCustom")]
    pub is_custom: bool,

    pub copyrights: Vec<Copyright>,

    #[serde(rename = "licenseBody")]
    pub license_body: Option<String>,
}

/* ---------- installer source ---------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubSource {
    pub owner: String,
    pub repo: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDriveSource {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InstallerSource {
    Direct {
        direct: String,
    },
    Booth {
        booth: String,
    },
    Github {
        github: GithubSource,
    },
    GoogleDrive {
        #[serde(rename = "GoogleDrive")]
        google_drive: GoogleDriveSource,
    },
}

/* ---------- installer actions ---------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum InstallerAction {
    #[serde(rename = "download")]
    Download {},

    #[serde(rename = "extract")]
    Extract {},

    #[serde(rename = "extract_sfx")]
    ExtractSfx {},

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

    #[serde(rename = "run_auo_setup")]
    RunAuoSetup { path: String },
}

/* ---------- installer ---------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installer {
    pub source: InstallerSource,
    pub install: Vec<InstallerAction>,
    pub uninstall: Vec<InstallerAction>,
}

/* ---------- version ---------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    pub path: String,

    #[serde(rename = "XXH3_128")]
    pub xxh3_128: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub release_date: String,
    pub file: Vec<VersionFile>,
}

/* ---------- image ---------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub thumbnail: Option<String>,

    #[serde(rename = "infoImg")]
    pub info_img: Option<Vec<String>>,
}

/* ---------- catalog entry ---------- */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogIndexEntry {
    pub id: String,
    pub name: String,

    #[serde(rename = "type")]
    pub entry_type: CatalogEntryType,

    pub summary: String,
    pub description: String,
    pub author: String,
    #[serde(rename = "originalAuthor")]
    pub original_author: Option<String>,

    #[serde(rename = "repoURL")]
    pub repo_url: String,

    pub licenses: Vec<License>,

    #[serde(rename = "niconiCommonsId")]
    pub niconi_commons_id: Option<String>,

    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub images: Vec<Image>,

    pub installer: Installer,
    pub version: Vec<Version>,

    // extra fields
    #[serde(rename = "latest-version")]
    pub latest_version: String,
}

#[derive(Debug, Clone)]
pub enum CatalogEntryType {
    AviUtl2,
    Output,
    Input,
    Filter,
    Common,
    Modification,
    Script,
    Other,

    Custom(String),
}
impl<'de> Deserialize<'de> for CatalogEntryType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "本体" => CatalogEntryType::AviUtl2,
            "出力プラグイン" => CatalogEntryType::Output,
            "入力プラグイン" => CatalogEntryType::Input,
            "フィルタプラグイン" => CatalogEntryType::Filter,
            "汎用プラグイン" => CatalogEntryType::Common,
            "MOD" => CatalogEntryType::Modification,
            "スクリプト" => CatalogEntryType::Script,
            "その他" => CatalogEntryType::Other,
            custom => CatalogEntryType::Custom(custom.to_string()),
        })
    }
}
impl Serialize for CatalogEntryType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            CatalogEntryType::AviUtl2 => "本体",
            CatalogEntryType::Output => "出力プラグイン",
            CatalogEntryType::Input => "入力プラグイン",
            CatalogEntryType::Filter => "フィルタプラグイン",
            CatalogEntryType::Common => "汎用プラグイン",
            CatalogEntryType::Modification => "MOD",
            CatalogEntryType::Script => "スクリプト",
            CatalogEntryType::Other => "その他",
            CatalogEntryType::Custom(custom) => custom.as_str(),
        };
        serializer.serialize_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::CatalogIndexEntry;

    const CATALOG_INDEX_URL: &str =
        "https://raw.githubusercontent.com/Neosku/aviutl2-catalog-data/refs/heads/main/index.json";

    #[test]
    fn actual_catalog_data_can_be_deserialized() -> anyhow::Result<()> {
        let response = ureq::get(CATALOG_INDEX_URL).call()?;
        let entries: Vec<CatalogIndexEntry> = response.into_body().read_json()?;

        assert!(!entries.is_empty());

        let first = &entries[0];
        assert!(!first.id.is_empty());
        assert!(!first.name.is_empty());
        assert!(!first.latest_version.is_empty());
        assert!(!first.licenses.is_empty());
        assert!(!first.version.is_empty());

        Ok(())
    }
}
