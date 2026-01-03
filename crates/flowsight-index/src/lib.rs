//! FlowSight Index
//!
//! Persistent storage for symbols and call graphs.

use flowsight_core::{Result, Error, FunctionDef, StructDef};
use std::path::Path;
use std::collections::HashMap;

/// Symbol index using SQLite
pub struct SymbolIndex {
    conn: rusqlite::Connection,
}

impl SymbolIndex {
    /// Create a new in-memory index
    pub fn new_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| Error::Index(e.to_string()))?;
        let index = Self { conn };
        index.init_tables()?;
        Ok(index)
    }

    /// Create a new index at the specified path
    pub fn new(path: &Path) -> Result<Self> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| Error::Index(e.to_string()))?;
        let index = Self { conn };
        index.init_tables()?;
        Ok(index)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS functions (
                name TEXT PRIMARY KEY,
                return_type TEXT,
                file TEXT,
                line INTEGER,
                is_callback INTEGER,
                callback_context TEXT,
                data TEXT
            );

            CREATE TABLE IF NOT EXISTS structs (
                name TEXT PRIMARY KEY,
                file TEXT,
                line INTEGER,
                data TEXT
            );

            CREATE TABLE IF NOT EXISTS calls (
                caller TEXT,
                callee TEXT,
                file TEXT,
                line INTEGER,
                call_type TEXT,
                PRIMARY KEY (caller, callee, line)
            );

            CREATE INDEX IF NOT EXISTS idx_functions_file ON functions(file);
            CREATE INDEX IF NOT EXISTS idx_calls_caller ON calls(caller);
            CREATE INDEX IF NOT EXISTS idx_calls_callee ON calls(callee);
        "#).map_err(|e| Error::Index(e.to_string()))?;
        Ok(())
    }

    /// Insert a function
    pub fn insert_function(&self, func: &FunctionDef) -> Result<()> {
        let data = serde_json::to_string(func)
            .map_err(|e| Error::Index(e.to_string()))?;
        
        let file = func.location.as_ref().map(|l| l.file.as_str()).unwrap_or("");
        let line = func.location.as_ref().map(|l| l.line as i64).unwrap_or(0);

        self.conn.execute(
            "INSERT OR REPLACE INTO functions (name, return_type, file, line, is_callback, callback_context, data) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                func.name,
                func.return_type,
                file,
                line,
                func.is_callback as i32,
                func.callback_context,
                data
            ],
        ).map_err(|e| Error::Index(e.to_string()))?;
        
        Ok(())
    }

    /// Search functions by name pattern
    pub fn search_functions(&self, pattern: &str) -> Result<Vec<FunctionDef>> {
        let mut stmt = self.conn.prepare(
            "SELECT data FROM functions WHERE name LIKE ?1"
        ).map_err(|e| Error::Index(e.to_string()))?;

        let pattern = format!("%{}%", pattern);
        let rows = stmt.query_map([&pattern], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        }).map_err(|e| Error::Index(e.to_string()))?;

        let mut functions = Vec::new();
        for row in rows {
            let data = row.map_err(|e| Error::Index(e.to_string()))?;
            if let Ok(func) = serde_json::from_str(&data) {
                functions.push(func);
            }
        }
        Ok(functions)
    }

    /// Get function by name
    pub fn get_function(&self, name: &str) -> Result<Option<FunctionDef>> {
        let mut stmt = self.conn.prepare(
            "SELECT data FROM functions WHERE name = ?1"
        ).map_err(|e| Error::Index(e.to_string()))?;

        let result = stmt.query_row([name], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        });

        match result {
            Ok(data) => {
                let func = serde_json::from_str(&data)
                    .map_err(|e| Error::Index(e.to_string()))?;
                Ok(Some(func))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Error::Index(e.to_string())),
        }
    }

    /// Get all callbacks
    pub fn get_callbacks(&self) -> Result<Vec<FunctionDef>> {
        let mut stmt = self.conn.prepare(
            "SELECT data FROM functions WHERE is_callback = 1"
        ).map_err(|e| Error::Index(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        }).map_err(|e| Error::Index(e.to_string()))?;

        let mut functions = Vec::new();
        for row in rows {
            let data = row.map_err(|e| Error::Index(e.to_string()))?;
            if let Ok(func) = serde_json::from_str(&data) {
                functions.push(func);
            }
        }
        Ok(functions)
    }

    /// Get callers of a function
    pub fn get_callers(&self, name: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT caller FROM calls WHERE callee = ?1"
        ).map_err(|e| Error::Index(e.to_string()))?;

        let rows = stmt.query_map([name], |row| {
            row.get(0)
        }).map_err(|e| Error::Index(e.to_string()))?;

        let mut callers = Vec::new();
        for row in rows {
            callers.push(row.map_err(|e| Error::Index(e.to_string()))?);
        }
        Ok(callers)
    }

    /// Get callees of a function
    pub fn get_callees(&self, name: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT callee FROM calls WHERE caller = ?1"
        ).map_err(|e| Error::Index(e.to_string()))?;

        let rows = stmt.query_map([name], |row| {
            row.get(0)
        }).map_err(|e| Error::Index(e.to_string()))?;

        let mut callees = Vec::new();
        for row in rows {
            callees.push(row.map_err(|e| Error::Index(e.to_string()))?);
        }
        Ok(callees)
    }
}

