use crate::types::ModuleType;
use serde::{Deserialize, Serialize};

/// A file hosting mirror server from the registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RegistryHost {
    pub alias: String,
    pub path: String,
    #[serde(default)]
    pub priority: f32,
    #[serde(default)]
    pub weight: f32,
}

/// Localized description entry for reading plans or multi-lingual modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LanguageDescription {
    pub lng: String,
    pub des: String,
}

/// Metadata describing an available downloadable module in the MyBible catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleDownloadEntry {
    #[serde(rename = "abr", default)]
    pub abbreviation: String,
    #[serde(rename = "fil", default)]
    pub file_stem: String,
    #[serde(rename = "des", default)]
    pub description: String,
    #[serde(rename = "lng")]
    pub language: Option<String>,
    #[serde(rename = "inf")]
    pub detailed_info: Option<String>,
    #[serde(rename = "siz")]
    pub size_str: Option<String>,
    #[serde(rename = "upd")]
    pub update_date: Option<String>,
    #[serde(rename = "cmt")]
    pub comment: Option<String>,
    #[serde(rename = "url", default)]
    pub urls: Vec<String>,
    #[serde(rename = "lds", default)]
    pub localized_descriptions: Vec<LanguageDescription>,
    #[serde(rename = "def", default)]
    pub is_default: bool,
    #[serde(rename = "hid", default)]
    pub is_hidden: bool,
    #[serde(rename = "reg")]
    pub region: Option<String>,
    #[serde(rename = "lic")]
    pub license: Option<String>,
    #[serde(rename = "lst")]
    pub bundle_file_list: Option<String>,
}

impl ModuleDownloadEntry {
    /// Detects the module type based on the file stem suffix.
    pub fn module_type(&self) -> ModuleType {
        let stem_lower = self.file_stem.to_lowercase();
        if stem_lower.ends_with(".dictionary") {
            ModuleType::Dictionary
        } else if stem_lower.ends_with(".commentaries") {
            ModuleType::Commentaries
        } else if stem_lower.ends_with(".crossreferences") {
            ModuleType::CrossReferences
        } else if stem_lower.ends_with(".subheadings") {
            ModuleType::Subheadings
        } else if stem_lower.ends_with(".plan") {
            ModuleType::ReadingPlan
        } else if stem_lower.ends_with(".devotions") {
            ModuleType::Devotions
        } else if stem_lower.ends_with(".bundle") {
            ModuleType::Bundle
        } else {
            ModuleType::Bible
        }
    }

    /// Resolve all available HTTP/HTTPS download URLs for this module, prioritizing fastest mirrors.
    pub fn resolve_urls(&self, hosts: &[RegistryHost]) -> Vec<String> {
        let mut list = Vec::new();

        // Sort hosts by priority ascending (1.0 is higher priority than 2.0)
        let mut sorted_hosts = hosts.to_vec();
        sorted_hosts.sort_by(|a, b| {
            a.priority
                .partial_cmp(&b.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for raw_url in &self.urls {
            if raw_url.starts_with('{') {
                if let Some(close_idx) = raw_url.find('}') {
                    let alias = &raw_url[1..close_idx];
                    let subpath = &raw_url[close_idx + 1..];
                    if let Some(host) = sorted_hosts.iter().find(|h| h.alias == alias) {
                        let direct = host.path.replace("%s", subpath);
                        list.push(direct);
                    }
                }
            } else if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
                list.push(raw_url.clone());
            }
        }

        list
    }

    /// Return expected target filename once downloaded and extracted.
    pub fn target_filename(&self) -> String {
        format!("{}.SQLite3", self.file_stem)
    }
}

/// Master catalog containing all available modules and host mirrors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleRegistry {
    pub version: Option<u64>,
    #[serde(default)]
    pub hosts: Vec<RegistryHost>,
    #[serde(rename = "downloads", default)]
    pub modules: Vec<ModuleDownloadEntry>,
}

impl ModuleRegistry {
    /// Parse registry from a JSON string.
    pub fn from_json_str(json: &str) -> crate::error::Result<Self> {
        let clean = json.trim_start_matches('\u{feff}').trim_start();
        let reg: ModuleRegistry = serde_json::from_str(clean)
            .map_err(|e| crate::error::MyBibleError::InvalidFormat(e.to_string()))?;
        Ok(reg)
    }

    /// Search modules by description or abbreviation (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&ModuleDownloadEntry> {
        let q = query.trim().to_lowercase();
        self.modules
            .iter()
            .filter(|m| {
                m.abbreviation.to_lowercase().contains(&q)
                    || m.description.to_lowercase().contains(&q)
                    || m.file_stem.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Filter modules by language code (e.g. "en", "es", "ru", "zh").
    pub fn filter_by_language(&self, lang: &str) -> Vec<&ModuleDownloadEntry> {
        let l = lang.trim().to_lowercase();
        self.modules
            .iter()
            .filter(|m| {
                m.language
                    .as_deref()
                    .map(|code| code.to_lowercase().starts_with(&l))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Filter modules by module type (Bible, Commentary, Dictionary, etc.).
    pub fn filter_by_type(&self, mtype: ModuleType) -> Vec<&ModuleDownloadEntry> {
        self.modules
            .iter()
            .filter(|m| m.module_type() == mtype)
            .collect()
    }

    /// Find a module by abbreviation.
    pub fn find_by_abbrev(
        &self,
        abbrev: &str,
        mtype: Option<ModuleType>,
    ) -> Option<&ModuleDownloadEntry> {
        self.modules.iter().find(|m| {
            m.abbreviation.eq_ignore_ascii_case(abbrev)
                && (mtype.is_none() || Some(m.module_type()) == mtype)
        })
    }
}
