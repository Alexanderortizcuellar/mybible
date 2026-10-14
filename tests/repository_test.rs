use mybible::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_parse_registry_and_search() {
    let cached_path = Path::new("cache/registry_extracted/registry.json");
    if !cached_path.exists() {
        println!("Skipping: cached registry.json not found");
        return;
    }

    let json_str = fs::read_to_string(cached_path).expect("Read registry.json");
    let registry = ModuleRegistry::from_json_str(&json_str).expect("Parse ModuleRegistry");

    println!("Total modules in registry: {}", registry.modules.len());
    assert!(registry.modules.len() > 4000);

    // Search for KJV
    let kjv_results = registry.search("KJV");
    println!("Found {} modules matching 'KJV'", kjv_results.len());
    assert!(!kjv_results.is_empty());

    // Search for Reina-Valera (Spanish)
    let spanish_bibles = registry.filter_by_language("es");
    println!("Found {} Spanish modules", spanish_bibles.len());
    assert!(!spanish_bibles.is_empty());

    // Find KJV+
    let kjv_plus = registry
        .find_by_abbrev("KJV+", Some(ModuleType::Bible))
        .expect("KJV+ must exist");
    assert_eq!(kjv_plus.abbreviation, "KJV+");
    assert_eq!(kjv_plus.module_type(), ModuleType::Bible);

    // Resolve URLs
    let resolved = kjv_plus.resolve_urls(&registry.hosts);
    println!("Resolved URLs for KJV+: {:?}", resolved);
    assert!(!resolved.is_empty());
    assert!(resolved[0].starts_with("http"));
}

#[test]
fn test_live_download_and_extract_plan() {
    let client = RepositoryClient::new();
    let dir = tempdir().unwrap();

    // Use a small reading plan module for fast test execution: 100EJ-p
    let entry = ModuleDownloadEntry {
        abbreviation: "100EJ-p".to_string(),
        file_stem: "100EJ-p.plan".to_string(),
        description: "100 days The Essential Jesus".to_string(),
        language: Some("en".to_string()),
        detailed_info: None,
        size_str: Some("3K".to_string()),
        update_date: Some("2016-12-23".to_string()),
        comment: None,
        urls: vec![
            "{mz}100EJ-p.plan".to_string(),
            "{inf}100EJ-p.plan".to_string(),
            "{1gb}100EJ-p.plan".to_string(),
        ],
        localized_descriptions: Vec::new(),
        is_default: false,
        is_hidden: false,
        ..Default::default()
    };

    let hosts = vec![
        RegistryHost {
            alias: "mz".to_string(),
            path: "https://mybible.zone/repository/modules/%s.zip".to_string(),
            priority: 1.0,
            weight: 100.0,
        },
        RegistryHost {
            alias: "inf".to_string(),
            path: "https://mybible.infoo.pro/m/%s.zip".to_string(),
            priority: 1.0,
            weight: 100.0,
        },
    ];

    let registry = ModuleRegistry {
        version: Some(1),
        hosts,
        modules: vec![entry.clone()],
    };

    println!("Downloading module '100EJ-p.plan'...");
    let extracted_path = client
        .download_module(&entry, &registry, dir.path())
        .expect("Download module");

    println!("Extracted to: {:?}", extracted_path);
    assert!(extracted_path.exists());

    // Verify it opens and reads correctly with ReadingPlanReader
    let plan_reader =
        ReadingPlanReader::open(&extracted_path, "100EJ-p").expect("Open reading plan");
    let max_days = plan_reader.max_days().expect("Max days");
    println!("Plan duration in days: {}", max_days);
    assert_eq!(max_days, 100);

    let day1_items = plan_reader.items_for_day(1).expect("Day 1 items");
    println!("Day 1 items count: {}", day1_items.len());
    assert!(!day1_items.is_empty());
}

#[test]
fn test_engine_download_and_load() {
    let client = RepositoryClient::new();
    let dir = tempdir().unwrap();

    let entry = ModuleDownloadEntry {
        abbreviation: "100EJ-p".to_string(),
        file_stem: "100EJ-p.plan".to_string(),
        description: "100 days The Essential Jesus".to_string(),
        language: Some("en".to_string()),
        urls: vec!["{mz}100EJ-p.plan".to_string()],
        ..Default::default()
    };

    let registry = ModuleRegistry {
        version: Some(1),
        hosts: vec![RegistryHost {
            alias: "mz".to_string(),
            path: "https://mybible.zone/repository/modules/%s.zip".to_string(),
            priority: 1.0,
            weight: 100.0,
        }],
        modules: vec![entry.clone()],
    };

    let mut engine = MyBibleEngine::new();
    let mod_type = engine
        .download_and_load(&client, &entry, &registry, dir.path())
        .expect("download and load");

    assert_eq!(mod_type, ModuleType::ReadingPlan);
    assert!(engine.reading_plans.contains_key("100EJ-p"));
}
