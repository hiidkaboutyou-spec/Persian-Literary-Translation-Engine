use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::glossary::{Glossary, GlossaryEntry};
use crate::{MemoryEntry, TranslationMemory};

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "memory persistence I/O error: {error}"),
            Self::Json(error) => write!(formatter, "memory persistence JSON error: {error}"),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

fn ensure_parent(path: &Path) -> Result<(), PersistenceError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn save_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), PersistenceError> {
    ensure_parent(path)?;
    let file = File::create(path)?;
    serde_json::to_writer_pretty(BufWriter::new(file), value)?;
    Ok(())
}

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T, PersistenceError> {
    let file = File::open(path)?;
    Ok(serde_json::from_reader(BufReader::new(file))?)
}

pub fn save_translation_memory(
    path: impl AsRef<Path>,
    memory: &TranslationMemory,
) -> Result<(), PersistenceError> {
    save_json(path.as_ref(), memory.entries())
}

pub fn load_translation_memory(
    path: impl AsRef<Path>,
) -> Result<TranslationMemory, PersistenceError> {
    let entries: Vec<MemoryEntry> = load_json(path.as_ref())?;
    let mut memory = TranslationMemory::new();
    for entry in entries {
        memory.add(entry);
    }
    Ok(memory)
}

pub fn save_glossary(path: impl AsRef<Path>, glossary: &Glossary) -> Result<(), PersistenceError> {
    save_json(path.as_ref(), glossary.entries())
}

pub fn load_glossary(path: impl AsRef<Path>) -> Result<Glossary, PersistenceError> {
    let entries: Vec<GlossaryEntry> = load_json(path.as_ref())?;
    let mut glossary = Glossary::default();
    for entry in entries {
        glossary.add(entry);
    }
    Ok(glossary)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("literary-engine-{name}-{nonce}.json"))
    }

    #[test]
    fn translation_memory_round_trips_json() {
        let path = temp_path("memory");
        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "softly whispered".into(),
            "آرام زمزمه کرد".into(),
            "dialogue".into(),
        ));

        save_translation_memory(&path, &memory).unwrap();
        let restored = load_translation_memory(&path).unwrap();
        fs::remove_file(path).ok();

        assert_eq!(restored.entries().len(), 1);
        assert_eq!(restored.entries()[0].translation, "آرام زمزمه کرد");
    }

    #[test]
    fn glossary_round_trips_json() {
        let path = temp_path("glossary");
        let mut glossary = Glossary::default();
        glossary.add(GlossaryEntry {
            source_term: "High Warlock".into(),
            preferred_translation: "جادوگر اعظم".into(),
            context: "title".into(),
        });

        save_glossary(&path, &glossary).unwrap();
        let restored = load_glossary(&path).unwrap();
        fs::remove_file(path).ok();

        assert_eq!(restored.entries().len(), 1);
        assert_eq!(restored.entries()[0].source_term, "High Warlock");
    }
}
