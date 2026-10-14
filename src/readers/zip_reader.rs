use crate::error::Result;
use crate::readers::file_detector::detect_module_file;
use crate::types::ModuleType;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Entry inside a zipped MyBible archive.
#[derive(Debug, Clone)]
pub struct ZippedModuleEntry {
    pub inner_path: String,
    pub abbreviation: String,
    pub module_type: ModuleType,
    pub size: u64,
}

/// Reader for zipped MyBible modules and `.bundle.zip` archives.
pub struct ZipModuleArchive {
    path: PathBuf,
    entries: Vec<ZippedModuleEntry>,
}

impl ZipModuleArchive {
    /// Opens and indexes the contents of a MyBible `.zip` archive.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref().to_path_buf();
        let file = File::open(&p)?;
        let mut archive = ZipArchive::new(file)?;

        // Determine base abbreviation and type from the ZIP file itself
        let zip_info = detect_module_file(&p);
        let base_abbrev = zip_info
            .as_ref()
            .map(|i| i.abbreviation.as_str())
            .unwrap_or("Module");

        let mut entries = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();

            // Ignore directories
            if file.is_dir() {
                continue;
            }

            let path_obj = Path::new(name);
            let file_name = match path_obj.file_name().and_then(|f| f.to_str()) {
                Some(f) => f,
                None => continue,
            };

            let name_lower = file_name.to_lowercase();
            if !name_lower.ends_with(".sqlite3") && !name_lower.ends_with(".sqlite") {
                continue;
            }

            // Check pre-4.8.0 convention: starts with "."
            if file_name.starts_with('.') {
                if file_name == ".SQLite3" || file_name == ".sqlite3" {
                    entries.push(ZippedModuleEntry {
                        inner_path: name.to_string(),
                        abbreviation: base_abbrev.to_string(),
                        module_type: ModuleType::Bible,
                        size: file.size(),
                    });
                } else if name_lower.starts_with(".commentaries.") {
                    entries.push(ZippedModuleEntry {
                        inner_path: name.to_string(),
                        abbreviation: base_abbrev.to_string(),
                        module_type: ModuleType::Commentaries,
                        size: file.size(),
                    });
                } else if name_lower.starts_with(".dictionary.") {
                    entries.push(ZippedModuleEntry {
                        inner_path: name.to_string(),
                        abbreviation: base_abbrev.to_string(),
                        module_type: ModuleType::Dictionary,
                        size: file.size(),
                    });
                } else if name_lower.starts_with(".subheadings.") {
                    entries.push(ZippedModuleEntry {
                        inner_path: name.to_string(),
                        abbreviation: base_abbrev.to_string(),
                        module_type: ModuleType::Subheadings,
                        size: file.size(),
                    });
                } else if name_lower.starts_with(".crossreferences.") {
                    entries.push(ZippedModuleEntry {
                        inner_path: name.to_string(),
                        abbreviation: base_abbrev.to_string(),
                        module_type: ModuleType::CrossReferences,
                        size: file.size(),
                    });
                }
            } else if let Some(info) = detect_module_file(path_obj) {
                // Post-4.8.0 convention: normal full filenames inside zip
                entries.push(ZippedModuleEntry {
                    inner_path: name.to_string(),
                    abbreviation: info.abbreviation,
                    module_type: info.module_type,
                    size: file.size(),
                });
            }
        }

        Ok(Self { path: p, entries })
    }

    pub fn entries(&self) -> &[ZippedModuleEntry] {
        &self.entries
    }

    /// Read raw bytes of an inner file from the archive.
    pub fn read_bytes(&self, inner_path: &str) -> Result<Vec<u8>> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut zip_file = archive.by_name(inner_path)?;
        let mut buf = Vec::with_capacity(zip_file.size() as usize);
        zip_file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Extract all files from this archive into a destination directory.
    pub fn extract_all(&self, dest_dir: impl AsRef<Path>) -> Result<()> {
        let file = File::open(&self.path)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(dest_dir.as_ref())?;
        Ok(())
    }
}
