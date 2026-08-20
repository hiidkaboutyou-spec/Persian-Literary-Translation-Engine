use rusqlite::{Connection, Result};

pub struct TranslationDatabase {
    connection: Connection,
}

impl TranslationDatabase {
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
        Ok(())
    }
}
