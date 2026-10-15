use crate::error::{MyBibleError, Result};
use crate::repository::models::{ModuleDownloadEntry, ModuleRegistry};
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const OFFICIAL_REGISTRY_URL: &str = "https://mybible.zone/repository/registry/registry.zip";

/// Client for communicating with MyBible module repositories and downloading modules.
pub struct RepositoryClient {
    agent: ureq::Agent,
}

impl Default for RepositoryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoryClient {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(10))
            .timeout_read(Duration::from_secs(60))
            .user_agent("MyBible/5.8.0 (Android; Rust Engine)")
            .build();
        Self { agent }
    }

    /// Fetches and parses the official MyBible module registry from the cloud or local disk cache.
    pub fn fetch_official_registry(&self) -> Result<ModuleRegistry> {
        let cache_path =
            dirs::home_dir().map(|h| h.join(".mybible").join("cache").join("registry.json"));
        if let Some(ref path) = cache_path {
            if path.exists() {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            // 3 days cache freshness
                            if elapsed.as_secs() < 86400 * 3 {
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    if let Ok(reg) = ModuleRegistry::from_json_str(&content) {
                                        return Ok(reg);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let reg = self.fetch_registry_zip(OFFICIAL_REGISTRY_URL)?;

        // Cache registry to disk for offline access and instant future loads
        if let Some(ref path) = cache_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json_str) = serde_json::to_string(&reg) {
                let _ = std::fs::write(path, json_str);
            }
        }

        Ok(reg)
    }

    /// Fetches a module registry from a given URL (supports both .zip and .json).
    pub fn fetch_registry(&self, url: &str) -> Result<ModuleRegistry> {
        if url.ends_with(".zip") {
            self.fetch_registry_zip(url)
        } else {
            self.fetch_registry_json(url)
        }
    }

    /// Fetches a `.zip` registry archive, extracts `registry.json` in-memory, and parses it.
    pub fn fetch_registry_zip(&self, url: &str) -> Result<ModuleRegistry> {
        let response = self.agent.get(url).call().map_err(|e| {
            MyBibleError::Http(format!("Failed to fetch registry from {}: {}", url, e))
        })?;

        let mut zip_bytes = Vec::new();
        response.into_reader().read_to_end(&mut zip_bytes)?;

        let reader = Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(reader)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with(".json") {
                let mut json_str = String::new();
                file.read_to_string(&mut json_str)?;
                return ModuleRegistry::from_json_str(&json_str);
            }
        }

        Err(MyBibleError::InvalidFormat(
            "registry.json not found inside registry.zip".to_string(),
        ))
    }

    /// Fetches a `.json` registry directly.
    pub fn fetch_registry_json(&self, url: &str) -> Result<ModuleRegistry> {
        let response = self.agent.get(url).call().map_err(|e| {
            MyBibleError::Http(format!("Failed to fetch registry JSON from {}: {}", url, e))
        })?;

        let mut json_str = String::new();
        response.into_reader().read_to_string(&mut json_str)?;

        ModuleRegistry::from_json_str(&json_str)
    }

    /// Downloads and extracts a module into the destination directory.
    ///
    /// Returns the path to the newly extracted `.SQLite3` module file.
    pub fn download_module(
        &self,
        entry: &ModuleDownloadEntry,
        registry: &ModuleRegistry,
        dest_dir: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let dest = dest_dir.as_ref();
        fs::create_dir_all(dest)?;

        let urls = entry.resolve_urls(&registry.hosts);
        if urls.is_empty() {
            return Err(MyBibleError::InvalidFormat(format!(
                "No download URLs available for module '{}'",
                entry.abbreviation
            )));
        }

        let mut downloaded_bytes = None;
        let mut last_err = None;

        for url in &urls {
            match self.agent.get(url).call() {
                Ok(resp) => {
                    let mut bytes = Vec::new();
                    if resp.into_reader().read_to_end(&mut bytes).is_ok() {
                        downloaded_bytes = Some(bytes);
                        break;
                    }
                }
                Err(e) => {
                    last_err = Some(e.to_string());
                }
            }
        }

        let zip_bytes = downloaded_bytes.ok_or_else(|| {
            MyBibleError::Http(format!(
                "Failed to download module '{}' from any mirror: {:?}",
                entry.abbreviation, last_err
            ))
        })?;

        // Extract all SQLite modules from zip
        let mut primary_path = None;
        let mut extracted_any = false;

        let reader = Cursor::new(&zip_bytes);
        let mut archive = zip::ZipArchive::new(reader)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let raw_name = file.name().to_string();

            // Extract file name without directories
            let bname = Path::new(&raw_name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&raw_name);

            let bname_lower = bname.to_lowercase();
            if bname_lower.ends_with(".sqlite3") || bname_lower.ends_with(".sqlite") {
                // Determine target file name based on entry.file_stem and bname
                let target_name = if bname.eq_ignore_ascii_case(".sqlite3") {
                    format!("{}.SQLite3", entry.file_stem)
                } else if bname.starts_with('.') {
                    // e.g. .commentaries.SQLite3 or .subheadings.SQLite3
                    let suffix = &bname[..bname.len().saturating_sub(8)]; // e.g. .commentaries
                    if entry
                        .file_stem
                        .to_lowercase()
                        .ends_with(&suffix.to_lowercase())
                    {
                        format!("{}.SQLite3", entry.file_stem)
                    } else {
                        format!("{}{}", entry.file_stem, bname)
                    }
                } else {
                    bname.to_string()
                };

                let target_path = dest.join(&target_name);
                let mut out_file = File::create(&target_path)?;
                std::io::copy(&mut file, &mut out_file)?;
                extracted_any = true;

                // Pick primary path: prefer one matching entry.target_filename()
                if primary_path.is_none()
                    || target_name.eq_ignore_ascii_case(&entry.target_filename())
                {
                    primary_path = Some(target_path);
                }
            }
        }

        if !extracted_any {
            // Bundle or alternative format: extract all files preserving relative paths
            let reader = Cursor::new(&zip_bytes);
            let mut archive = zip::ZipArchive::new(reader)?;
            archive.extract(dest)?;
            return Ok(dest.join(entry.target_filename()));
        }

        primary_path.ok_or_else(|| {
            MyBibleError::InvalidFormat(format!(
                "No SQLite module database found inside zip for '{}'",
                entry.abbreviation
            ))
        })
    }
}
