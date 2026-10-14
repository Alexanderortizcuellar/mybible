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

    /// Fetches and parses the official MyBible module registry from the cloud.
    pub fn fetch_official_registry(&self) -> Result<ModuleRegistry> {
        self.fetch_registry_zip(OFFICIAL_REGISTRY_URL)
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

        // Extract the SQLite module from zip
        let target_filename = entry.target_filename();
        let target_path = dest.join(&target_filename);

        let reader = Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(reader)?;

        let mut extracted = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            // Look for SQLite3 file inside
            if name.ends_with(".SQLite3") || name.ends_with(".sqlite3") || name.ends_with(".sqlite")
            {
                let mut out_file = File::create(&target_path)?;
                std::io::copy(&mut file, &mut out_file)?;
                extracted = true;
                break;
            }
        }

        if !extracted {
            // Bundle or alternative format: extract all files preserving relative paths
            let reader = Cursor::new(archive.into_inner().into_inner());
            let mut archive = zip::ZipArchive::new(reader)?;
            archive.extract(dest)?;
            return Ok(target_path);
        }

        Ok(target_path)
    }
}
