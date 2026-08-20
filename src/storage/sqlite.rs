use rusqlite::{Connection, Result};

use super::traits::{GlossaryStore, TranslationMemoryStore};

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
                context TEXT
            )",
            [],
        )?;

        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS glossary_entries (
                id INTEGER PRIMARY KEY,
                term TEXT NOT NULL,
                translation TEXT NOT NULL,
                notes TEXT
            )",
            [],
        )?;

        Ok(())
    }
}

// The storage adapter owns database details.
// Translation logic stays independent from SQLite.
