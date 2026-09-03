//! Deterministic analysis cache.
//!
//! A unit result is reusable only when every material input matches: analysis
//! schema, prompt version, provider identity, model identity, planner/config
//! fingerprint, canon fingerprint, and the unit content fingerprint. Keys
//! never include timestamps or credentials.

use crate::models::{AdvancedAnalysisError, AdvancedLiteraryFinding};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const ANALYSIS_CACHE_SCHEMA_VERSION: u32 = 1;
/// Hard cap on persisted entries to bound memory and disk growth.
const MAX_CACHE_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CachedUnitResult {
    pub key: String,
    pub unit_id: String,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub findings: Vec<AdvancedLiteraryFinding>,
    pub stored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisCacheFile {
    pub schema_version: u32,
    pub entries: Vec<CachedUnitResult>,
}

pub struct AnalysisCache {
    path: Option<std::path::PathBuf>,
    entries: BTreeMap<String, CachedUnitResult>,
}

impl AnalysisCache {
    pub fn new(path: Option<std::path::PathBuf>) -> Self {
        let entries = match &path {
            Some(path) if path.is_file() => std::fs::read_to_string(path)
                .ok()
                .and_then(|text| serde_json::from_str::<AnalysisCacheFile>(&text).ok())
                .map(|file| {
                    file.entries
                        .into_iter()
                        .map(|entry| (entry.key.clone(), entry))
                        .collect::<BTreeMap<_, _>>()
                })
                .unwrap_or_default(),
            _ => BTreeMap::new(),
        };
        Self { path, entries }
    }

    pub fn is_enabled(&self) -> bool {
        self.path.is_some()
    }

    pub fn get(&self, key: &str) -> Option<&CachedUnitResult> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, result: CachedUnitResult) {
        self.entries.insert(result.key.clone(), result);
    }

    pub fn cached_count(&self) -> usize {
        self.entries.len()
    }

    pub fn persist(&self) -> Result<(), AdvancedAnalysisError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    AdvancedAnalysisError::CacheIo(format!(
                        "failed to create cache directory {}: {error}",
                        parent.display()
                    ))
                })?;
            }
        }
        // Keep the newest entries by write time; iteration order of the map is
        // key order, and keys are content hashes, so cap deterministically.
        let mut entries = self.entries.values().cloned().collect::<Vec<_>>();
        if entries.len() > MAX_CACHE_ENTRIES {
            entries.sort_by_key(|entry| std::cmp::Reverse(entry.stored_at));
            entries.truncate(MAX_CACHE_ENTRIES);
            entries.sort_by(|left, right| left.key.cmp(&right.key));
        }
        let file = AnalysisCacheFile {
            schema_version: ANALYSIS_CACHE_SCHEMA_VERSION,
            entries,
        };
        let serialized = serde_json::to_string_pretty(&file).map_err(|error| {
            AdvancedAnalysisError::CacheIo(format!("failed to serialize cache: {error}"))
        })?;
        std::fs::write(path, serialized).map_err(|error| {
            AdvancedAnalysisError::CacheIo(format!(
                "failed to write cache {}: {error}",
                path.display()
            ))
        })
    }
}

/// Stable cache key for one unit: schema + prompt + provider + model +
/// planner/config fingerprint + canon fingerprint + unit identity.
pub fn cache_key(
    schema_version: u32,
    prompt_version: &str,
    provider: &str,
    model: &str,
    configuration_fingerprint: &str,
    canon_fingerprint: &str,
    unit_id: &str,
) -> String {
    let identity = format!(
        "{schema_version}\0{prompt_version}\0{provider}\0{model}\0{configuration_fingerprint}\0{canon_fingerprint}\0{unit_id}"
    );
    format!("cache-{:x}", Sha256::digest(identity.as_bytes()))
}

/// Fingerprint of the approved canon context actually supplied to analysis.
/// Sorted serialization keeps the fingerprint independent of input ordering.
pub fn canon_fingerprint(character_names: &[String], glossary_terms: &[String]) -> String {
    let mut names = character_names.to_vec();
    names.sort();
    let mut terms = glossary_terms.to_vec();
    terms.sort();
    let identity = format!("{:?}\0{:?}", names, terms);
    format!("{:x}", Sha256::digest(identity.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_stable_and_sensitive_to_material_inputs() {
        let base = cache_key(1, "v1", "mock", "m", "cfg", "canon", "unit-1");
        assert_eq!(
            base,
            cache_key(1, "v1", "mock", "m", "cfg", "canon", "unit-1")
        );
        assert_ne!(
            base,
            cache_key(1, "v2", "mock", "m", "cfg", "canon", "unit-1")
        );
        assert_ne!(
            base,
            cache_key(1, "v1", "openai", "m", "cfg", "canon", "unit-1")
        );
        assert_ne!(
            base,
            cache_key(1, "v1", "mock", "m", "cfg", "canon", "unit-2")
        );
    }

    #[test]
    fn cache_round_trips_through_a_temp_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("analysis-cache.json");
        let mut cache = AnalysisCache::new(Some(path.clone()));
        let entry = CachedUnitResult {
            key: "cache-key".to_string(),
            unit_id: "unit-1".to_string(),
            provider: "mock".to_string(),
            model: "mock-v1".to_string(),
            prompt_version: "v1".to_string(),
            findings: Vec::new(),
            stored_at: Utc::now(),
        };
        cache.insert(entry);
        cache.persist().unwrap();

        let reloaded = AnalysisCache::new(Some(path));
        assert!(reloaded.get("cache-key").is_some());
        assert_eq!(reloaded.get("cache-key").unwrap().unit_id, "unit-1");
    }
}
