use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

pub mod review_store;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectManifest {
    pub schema_version: u32,
    pub project_name: String,
    pub source_path: String,
    pub target_language: String,
    pub provider: ProviderConfig,
    pub memory: MemoryConfig,
    pub export: ExportConfig,
    pub chapters: Vec<ChapterRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderConfig {
    pub name: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryConfig {
    pub translation_memory_path: Option<String>,
    pub glossary_path: Option<String>,
    pub character_bible_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportConfig {
    pub output_dir: String,
    pub docx_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChapterRecord {
    pub index: usize,
    pub title: String,
    pub state: ChapterState,
    pub source_fingerprint: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChapterState {
    Pending,
    Translating,
    ReviewNeeded,
    Approved,
    Blocked,
    Exported,
}

impl ProjectManifest {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(
        project_name: impl Into<String>,
        source_path: impl Into<String>,
        target_language: impl Into<String>,
        output_dir: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            project_name: project_name.into(),
            source_path: source_path.into(),
            target_language: target_language.into(),
            provider: ProviderConfig {
                name: "auto".to_string(),
                model: None,
            },
            memory: MemoryConfig {
                translation_memory_path: None,
                glossary_path: None,
                character_bible_path: None,
            },
            export: ExportConfig {
                output_dir: output_dir.into(),
                docx_filename: "manuscript.docx".to_string(),
            },
            chapters: Vec::new(),
        }
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        fs::write(path, json)
    }

    pub fn load_json(path: impl AsRef<Path>) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&json)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        if manifest.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "unsupported project schema version {}; expected {}",
                    manifest.schema_version,
                    Self::CURRENT_SCHEMA_VERSION
                ),
            ));
        }
        Ok(manifest)
    }

    pub fn chapter_counts(&self) -> ChapterCounts {
        let mut counts = ChapterCounts::default();
        for chapter in &self.chapters {
            match chapter.state {
                ChapterState::Pending => counts.pending += 1,
                ChapterState::Translating => counts.translating += 1,
                ChapterState::ReviewNeeded => counts.review_needed += 1,
                ChapterState::Approved => counts.approved += 1,
                ChapterState::Blocked => counts.blocked += 1,
                ChapterState::Exported => counts.exported += 1,
            }
        }
        counts
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChapterCounts {
    pub pending: usize,
    pub translating: usize,
    pub review_needed: usize,
    pub approved: usize,
    pub blocked: usize,
    pub exported: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("literary-engine-{name}-{nonce}.json"))
    }

    #[test]
    fn manifest_round_trips_with_persian_metadata() {
        let path = temp_path("manifest");
        let mut manifest = ProjectManifest::new("رمان من", "book.epub", "fa", "output");
        manifest.chapters.push(ChapterRecord {
            index: 1,
            title: "فصل اول".to_string(),
            state: ChapterState::ReviewNeeded,
            source_fingerprint: Some("abc123".to_string()),
            last_error: None,
        });

        manifest.save_json(&path).unwrap();
        let loaded = ProjectManifest::load_json(&path).unwrap();
        assert_eq!(loaded, manifest);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn chapter_counts_track_workflow_state() {
        let mut manifest = ProjectManifest::new("x", "book.txt", "fa", "out");
        manifest.chapters = vec![
            ChapterRecord {
                index: 1,
                title: "1".into(),
                state: ChapterState::Approved,
                source_fingerprint: None,
                last_error: None,
            },
            ChapterRecord {
                index: 2,
                title: "2".into(),
                state: ChapterState::Pending,
                source_fingerprint: None,
                last_error: None,
            },
            ChapterRecord {
                index: 3,
                title: "3".into(),
                state: ChapterState::Blocked,
                source_fingerprint: None,
                last_error: Some("provider timeout".into()),
            },
        ];
        let counts = manifest.chapter_counts();
        assert_eq!(counts.approved, 1);
        assert_eq!(counts.pending, 1);
        assert_eq!(counts.blocked, 1);
    }

    #[test]
    fn rejects_unknown_schema_versions() {
        let path = temp_path("future-schema");
        fs::write(&path, r#"{"schema_version":99,"project_name":"x","source_path":"x","target_language":"fa","provider":{"name":"auto","model":null},"memory":{"translation_memory_path":null,"glossary_path":null,"character_bible_path":null},"export":{"output_dir":"out","docx_filename":"manuscript.docx"},"chapters":[]}"#).unwrap();
        let err = ProjectManifest::load_json(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        let _ = fs::remove_file(path);
    }
}
