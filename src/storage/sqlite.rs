use anyhow::Result;
use rusqlite::{params, Connection};

use super::traits::{GlossaryEntry, GlossaryStore, TranslationMemoryEntry, TranslationMemoryStore};

pub struct SqliteStorage {
    connection: Connection,
}

impl SqliteStorage {
    pub fn open(path: &str) -> Result<Self> {
        let connection = Connection::open(path)?;
        Ok(Self { connection })
    }

    pub fn initialize(&self) -> Result<()> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS translation_memory (
                id INTEGER PRIMARY KEY,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                project_id TEXT NOT NULL
            )",
            [],
        )?;

        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS glossary_entries (
                id INTEGER PRIMARY KEY,
                term TEXT NOT NULL,
                translation TEXT NOT NULL,
                project_id TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }
}

impl TranslationMemoryStore for SqliteStorage {
    fn save(&self, entry: TranslationMemoryEntry) -> Result<()> {
        self.connection.execute(
            "INSERT INTO translation_memory (source, target, project_id) VALUES (?1, ?2, ?3)",
            params![entry.source, entry.translation, entry.project_id],
        )?;
        Ok(())
    }

    fn search(&self, query: &str) -> Result<Vec<TranslationMemoryEntry>> {
        let mut statement = self.connection.prepare(
            "SELECT source, target, project_id FROM translation_memory WHERE source LIKE ?1",
        )?;

        let rows = statement.query_map(params![format!("%{}%", query)], |row| {
            Ok(TranslationMemoryEntry {
                source: row.get(0)?,
                translation: row.get(1)?,
                project_id: row.get(2)?,
            })
        })?;

        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

impl GlossaryStore for SqliteStorage {
    fn add(&self, entry: GlossaryEntry) -> Result<()> {
        self.connection.execute(
            "INSERT INTO glossary_entries (term, translation, project_id) VALUES (?1, ?2, ?3)",
            params![entry.term, entry.preferred_translation, entry.project_id],
        )?;
        Ok(())
    }

    fn lookup(&self, term: &str) -> Result<Option<GlossaryEntry>> {
        let mut statement = self.connection.prepare(
            "SELECT term, translation, project_id FROM glossary_entries WHERE term = ?1 LIMIT 1",
        )?;

        let mut rows = statement.query(params![term])?;

        if let Some(row) = rows.next()? {
            return Ok(Some(GlossaryEntry {
                term: row.get(0)?,
                preferred_translation: row.get(1)?,
                project_id: row.get(2)?,
            }));
        }

        Ok(None)
    }
}
