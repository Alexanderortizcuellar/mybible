use crate::types::ModuleType;
use std::path::Path;

/// Information detected from a MyBible module filename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFileInfo {
    pub abbreviation: String,
    pub module_type: ModuleType,
    pub is_zipped: bool,
}

/// Detects the module abbreviation and type from a file path.
///
/// Supported patterns according to MyBible specification:
/// - `<abbrev>.SQLite3` -> Bible
/// - `<abbrev>.dictionary.SQLite3` -> Dictionary
/// - `<abbrev>.subheadings.SQLite3` -> Subheadings
/// - `<abbrev>.crossreferences.SQLite3` -> Cross References
/// - `<abbrev>.commentaries.SQLite3` -> Commentaries
/// - `<abbrev>.plan.SQLite3` -> Reading Plan
/// - `<abbrev>.devotions.SQLite3` -> Daily Devotions
/// - `<abbrev>.bundle.zip` -> Bundle
/// - `<abbrev>[.suffix].zip` -> Zipped module
///
/// Also handles alternative common extensions (.sqlite, .bbl.mybible, etc.).
pub fn detect_module_file(path: &Path) -> Option<ModuleFileInfo> {
    let file_name = path.file_name()?.to_str()?;
    let name_lower = file_name.to_lowercase();

    let is_zipped = name_lower.ends_with(".zip");
    let is_sqlite = name_lower.ends_with(".sqlite3")
        || name_lower.ends_with(".sqlite")
        || name_lower.ends_with(".mybible");

    if !is_zipped && !is_sqlite {
        return None;
    }

    // Strip trailing extension
    let stem = if is_zipped {
        &file_name[..file_name.len() - 4]
    } else if name_lower.ends_with(".sqlite3") {
        &file_name[..file_name.len() - 8]
    } else if name_lower.ends_with(".sqlite") {
        &file_name[..file_name.len() - 7]
    } else if name_lower.ends_with(".mybible") {
        &file_name[..file_name.len() - 8]
    } else {
        file_name
    };

    let stem_lower = stem.to_lowercase();

    // Check module type suffix
    let (abbreviation, module_type) = if stem_lower.ends_with(".dictionary") {
        (&stem[..stem.len() - 11], ModuleType::Dictionary)
    } else if stem_lower.ends_with(".subheadings") {
        (&stem[..stem.len() - 12], ModuleType::Subheadings)
    } else if stem_lower.ends_with(".crossreferences") {
        (&stem[..stem.len() - 16], ModuleType::CrossReferences)
    } else if stem_lower.ends_with(".commentaries") {
        (&stem[..stem.len() - 13], ModuleType::Commentaries)
    } else if stem_lower.ends_with(".plan") {
        (&stem[..stem.len() - 5], ModuleType::ReadingPlan)
    } else if stem_lower.ends_with(".devotions") {
        (&stem[..stem.len() - 10], ModuleType::Devotions)
    } else if stem_lower.ends_with(".bundle") {
        (&stem[..stem.len() - 7], ModuleType::Bundle)
    } else if stem_lower.ends_with(".bbl") {
        (&stem[..stem.len() - 4], ModuleType::Bible)
    } else {
        (stem, ModuleType::Bible)
    };

    Some(ModuleFileInfo {
        abbreviation: abbreviation.to_string(),
        module_type,
        is_zipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detection() {
        let cases = [
            ("KJV+.SQLite3", "KJV+", ModuleType::Bible, false),
            ("RST.SQLite3", "RST", ModuleType::Bible, false),
            (
                "Strong.dictionary.SQLite3",
                "Strong",
                ModuleType::Dictionary,
                false,
            ),
            (
                "KJV-s.subheadings.SQLite3",
                "KJV-s",
                ModuleType::Subheadings,
                false,
            ),
            (
                "OBX.crossreferences.SQLite3",
                "OBX",
                ModuleType::CrossReferences,
                false,
            ),
            (
                "RCAS.commentaries.SQLite3",
                "RCAS",
                ModuleType::Commentaries,
                false,
            ),
            (
                "AGV-p.plan.SQLite3",
                "AGV-p",
                ModuleType::ReadingPlan,
                false,
            ),
            (
                "DCF-d.devotions.SQLite3",
                "DCF-d",
                ModuleType::Devotions,
                false,
            ),
            ("Aramaic.bundle.zip", "Aramaic", ModuleType::Bundle, true),
            ("РСП.zip", "РСП", ModuleType::Bible, true),
            (
                "CBTEL.dictionary.zip",
                "CBTEL",
                ModuleType::Dictionary,
                true,
            ),
        ];

        for (filename, expected_abbrev, expected_type, expected_zipped) in cases {
            let path = PathBuf::from(filename);
            let info =
                detect_module_file(&path).unwrap_or_else(|| panic!("Failed on {}", filename));
            assert_eq!(info.abbreviation, expected_abbrev);
            assert_eq!(info.module_type, expected_type);
            assert_eq!(info.is_zipped, expected_zipped);
        }
    }
}
