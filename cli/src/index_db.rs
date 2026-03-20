//! SQLite-backed cross-file index database
//!
//! Stores symbols, call edges, and async handlers extracted from source files.
//! Designed for Linux kernel scale (30,000+ files).

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use std::path::Path;

/// Row from the symbols table
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SymbolRow {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub file_subsystem: Option<String>,
    pub line_start: u32,
    pub line_end: u32,
    pub is_exported: bool,
    pub signature: Option<String>,
}

/// Row from the calls table
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CallRow {
    pub caller_name: String,
    pub caller_file: String,
    pub callee_name: String,
    pub call_line: u32,
    pub is_indirect: bool,
}

/// Index statistics
#[derive(Debug, Clone, Default)]
pub struct IndexDbStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_functions: usize,
    pub total_structs: usize,
    pub total_macros: usize,
    pub total_callbacks: usize,
    pub total_async_handlers_count: usize,
    pub total_calls: usize,
    pub subsystem_breakdown: Vec<(String, usize)>,
    pub top_callees: Vec<(String, usize)>,
    pub top_callers: Vec<(String, usize)>,
}

/// SQLite index database
pub struct IndexDb {
    conn: Connection,
}

impl IndexDb {
    /// Open or create an index database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

        let db = Self { conn };
        db.initialize_schema()?;
        db.configure_pragmas()?;
        Ok(db)
    }

    /// Open a read-only connection (for queries)
    pub fn open_readonly(path: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("Database not found at {}", path.display()))?;
        let db = Self { conn };
        db.configure_pragmas()?;
        Ok(db)
    }

    /// Configure SQLite for bulk-insert performance
    fn configure_pragmas(&self) -> Result<()> {
        self.conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -64000;
             PRAGMA temp_store = MEMORY;",
        )?;
        Ok(())
    }

    /// Create tables and indices
    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                subsystem TEXT,
                last_modified INTEGER,
                hash TEXT
            );

            CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_id INTEGER REFERENCES files(id),
                line_start INTEGER,
                line_end INTEGER,
                is_exported BOOLEAN DEFAULT FALSE,
                signature TEXT
            );

            CREATE TABLE IF NOT EXISTS calls (
                caller_id INTEGER REFERENCES symbols(id),
                callee_name TEXT NOT NULL,
                call_line INTEGER,
                is_indirect BOOLEAN DEFAULT FALSE
            );

            CREATE TABLE IF NOT EXISTS async_handlers (
                symbol_id INTEGER REFERENCES symbols(id),
                mechanism TEXT,
                can_sleep BOOLEAN
            );

            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);
            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_id);
            CREATE INDEX IF NOT EXISTS idx_calls_callee ON calls(callee_name);
            CREATE INDEX IF NOT EXISTS idx_calls_caller ON calls(caller_id);
            CREATE INDEX IF NOT EXISTS idx_files_subsystem ON files(subsystem);
            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);",
        )?;
        Ok(())
    }

    /// Begin a transaction for batch inserts
    pub fn begin_transaction(&self) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// Commit a transaction
    pub fn commit_transaction(&self) -> Result<()> {
        self.conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Insert or update a file record, returning its id
    pub fn upsert_file(
        &self,
        path: &str,
        subsystem: Option<&str>,
        last_modified: i64,
        hash: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO files (path, subsystem, last_modified, hash)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET
                subsystem = excluded.subsystem,
                last_modified = excluded.last_modified,
                hash = excluded.hash",
            params![path, subsystem, last_modified, hash],
        )?;

        let file_id: i64 = self.conn.query_row(
            "SELECT id FROM files WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )?;

        Ok(file_id)
    }

    /// Remove all symbols and calls for a given file
    pub fn clear_file_symbols(&self, file_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM async_handlers WHERE symbol_id IN
             (SELECT id FROM symbols WHERE file_id = ?1)",
            params![file_id],
        )?;
        self.conn.execute(
            "DELETE FROM calls WHERE caller_id IN
             (SELECT id FROM symbols WHERE file_id = ?1)",
            params![file_id],
        )?;
        self.conn.execute(
            "DELETE FROM symbols WHERE file_id = ?1",
            params![file_id],
        )?;
        Ok(())
    }

    /// Insert a symbol, returning its id
    pub fn insert_symbol(
        &self,
        name: &str,
        kind: &str,
        file_id: i64,
        line_start: u32,
        line_end: u32,
        is_exported: bool,
        signature: Option<&str>,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO symbols (name, kind, file_id, line_start, line_end, is_exported, signature)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, kind, file_id, line_start, line_end, is_exported, signature],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert a call edge
    pub fn insert_call(
        &self,
        caller_id: i64,
        callee_name: &str,
        call_line: u32,
        is_indirect: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO calls (caller_id, callee_name, call_line, is_indirect)
             VALUES (?1, ?2, ?3, ?4)",
            params![caller_id, callee_name, call_line, is_indirect],
        )?;
        Ok(())
    }

    /// Insert an async handler record
    pub fn insert_async_handler(
        &self,
        symbol_id: i64,
        mechanism: &str,
        can_sleep: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO async_handlers (symbol_id, mechanism, can_sleep)
             VALUES (?1, ?2, ?3)",
            params![symbol_id, mechanism, can_sleep],
        )?;
        Ok(())
    }

    /// Check if a file needs reindexing by comparing hash
    pub fn file_needs_reindex(&self, path: &str, current_hash: &str) -> Result<bool> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT hash FROM files WHERE path = ?1",
                params![path],
                |row| row.get(0),
            )
            .ok();

        match result {
            Some(stored_hash) => Ok(stored_hash != current_hash),
            None => Ok(true),
        }
    }

    /// Query all callers of a symbol across the entire index
    pub fn query_callers(&self, symbol_name: &str) -> Result<Vec<CallRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.name, f.path, c.callee_name, c.call_line, c.is_indirect
             FROM calls c
             JOIN symbols s ON c.caller_id = s.id
             JOIN files f ON s.file_id = f.id
             WHERE c.callee_name = ?1
             ORDER BY f.path, c.call_line",
        )?;

        let rows = stmt
            .query_map(params![symbol_name], |row| {
                Ok(CallRow {
                    caller_name: row.get(0)?,
                    caller_file: row.get(1)?,
                    callee_name: row.get(2)?,
                    call_line: row.get(3)?,
                    is_indirect: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Query what a function calls across all indexed files
    pub fn query_callees(&self, caller_name: &str) -> Result<Vec<CallRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.name, f.path, c.callee_name, c.call_line, c.is_indirect
             FROM calls c
             JOIN symbols s ON c.caller_id = s.id
             JOIN files f ON s.file_id = f.id
             WHERE s.name = ?1
             ORDER BY c.call_line",
        )?;

        let rows = stmt
            .query_map(params![caller_name], |row| {
                Ok(CallRow {
                    caller_name: row.get(0)?,
                    caller_file: row.get(1)?,
                    callee_name: row.get(2)?,
                    call_line: row.get(3)?,
                    is_indirect: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Query callers grouped by subsystem
    pub fn query_callers_by_subsystem(&self, symbol_name: &str) -> Result<Vec<(String, Vec<CallRow>)>> {
        let callers = self.query_callers(symbol_name)?;
        let mut by_subsystem: std::collections::HashMap<String, Vec<CallRow>> = std::collections::HashMap::new();

        for caller in callers {
            let subsystem = detect_subsystem(&caller.caller_file)
                .unwrap_or_else(|| "other".to_string());
            by_subsystem.entry(subsystem).or_default().push(caller);
        }

        let mut result: Vec<(String, Vec<CallRow>)> = by_subsystem.into_iter().collect();
        result.sort_by(|a, b| b.1.len().cmp(&a.1.len())); // Sort by count descending
        Ok(result)
    }

    /// Query a symbol by name, optionally filtered by kind and subsystem
    pub fn query_symbol(
        &self,
        name: &str,
        kind_filter: Option<&str>,
        subsystem_filter: Option<&str>,
    ) -> Result<Vec<SymbolRow>> {
        let base = "SELECT s.id, s.name, s.kind, f.path, f.subsystem,
                    s.line_start, s.line_end, s.is_exported, s.signature
             FROM symbols s
             JOIN files f ON s.file_id = f.id
             WHERE s.name = ?1";

        let rows = match (kind_filter, subsystem_filter) {
            (None, None) => {
                let mut stmt = self.conn.prepare(base)?;
                collect_symbol_rows(&mut stmt, params![name])?
            }
            (Some(kind), None) => {
                let sql = format!("{} AND s.kind = ?2", base);
                let mut stmt = self.conn.prepare(&sql)?;
                collect_symbol_rows(&mut stmt, params![name, kind])?
            }
            (None, Some(sub)) => {
                let sql = format!("{} AND f.subsystem = ?2", base);
                let mut stmt = self.conn.prepare(&sql)?;
                collect_symbol_rows(&mut stmt, params![name, sub])?
            }
            (Some(kind), Some(sub)) => {
                let sql = format!("{} AND s.kind = ?2 AND f.subsystem = ?3", base);
                let mut stmt = self.conn.prepare(&sql)?;
                collect_symbol_rows(&mut stmt, params![name, kind, sub])?
            }
        };

        Ok(rows)
    }

    /// Query symbols by partial name match (LIKE), optionally filtered by kind
    pub fn query_symbol_like(
        &self,
        pattern: &str,
        kind_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SymbolRow>> {
        let like_pattern = format!("%{}%", pattern);

        let base = "SELECT s.id, s.name, s.kind, f.path, f.subsystem,
                    s.line_start, s.line_end, s.is_exported, s.signature
             FROM symbols s
             JOIN files f ON s.file_id = f.id
             WHERE s.name LIKE ?1";

        let rows = match kind_filter {
            None => {
                let sql = format!("{} ORDER BY s.name LIMIT ?2", base);
                let mut stmt = self.conn.prepare(&sql)?;
                collect_symbol_rows(&mut stmt, params![like_pattern, limit])?
            }
            Some(kind) => {
                let sql = format!("{} AND s.kind = ?2 ORDER BY s.name LIMIT ?3", base);
                let mut stmt = self.conn.prepare(&sql)?;
                collect_symbol_rows(&mut stmt, params![like_pattern, kind, limit])?
            }
        };

        Ok(rows)
    }

    /// Count callers for a given symbol name
    pub fn count_callers(&self, symbol_name: &str) -> Result<usize> {
        self.count_query_param(
            "SELECT COUNT(*) FROM calls WHERE callee_name = ?1",
            symbol_name,
        )
    }

    /// Get index statistics
    pub fn stats(&self) -> Result<IndexDbStats> {
        let total_files = self.count_query("SELECT COUNT(*) FROM files")?;
        let total_symbols = self.count_query("SELECT COUNT(*) FROM symbols")?;
        let total_functions =
            self.count_query("SELECT COUNT(*) FROM symbols WHERE kind = 'function'")?;
        let total_structs =
            self.count_query("SELECT COUNT(*) FROM symbols WHERE kind = 'struct'")?;
        let total_macros =
            self.count_query("SELECT COUNT(*) FROM symbols WHERE kind = 'macro'")?;
        let total_callbacks =
            self.count_query("SELECT COUNT(*) FROM symbols WHERE kind = 'callback'")?;
        let total_async_handlers_count =
            self.count_query("SELECT COUNT(*) FROM async_handlers")?;
        let total_calls = self.count_query("SELECT COUNT(*) FROM calls")?;

        let subsystem_breakdown = self.query_subsystem_breakdown()?;
        let top_callees = self.query_top_n(
            "SELECT callee_name, COUNT(*) as cnt FROM calls
             GROUP BY callee_name ORDER BY cnt DESC LIMIT ?1",
            10,
        )?;
        let top_callers = self.query_top_n(
            "SELECT s.name, COUNT(*) as cnt FROM calls c
             JOIN symbols s ON c.caller_id = s.id
             GROUP BY s.name ORDER BY cnt DESC LIMIT ?1",
            10,
        )?;

        Ok(IndexDbStats {
            total_files,
            total_symbols,
            total_functions,
            total_structs,
            total_macros,
            total_callbacks,
            total_async_handlers_count,
            total_calls,
            subsystem_breakdown,
            top_callees,
            top_callers,
        })
    }

    /// Run a simple COUNT query
    fn count_query(&self, sql: &str) -> Result<usize> {
        let count: usize = self.conn.query_row(sql, [], |r| r.get(0))?;
        Ok(count)
    }

    /// Run a COUNT query with one string parameter
    fn count_query_param(&self, sql: &str, param: &str) -> Result<usize> {
        let count: usize = self.conn.query_row(sql, params![param], |r| r.get(0))?;
        Ok(count)
    }

    /// Get per-subsystem file counts
    fn query_subsystem_breakdown(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(subsystem, '(unknown)'), COUNT(*)
             FROM files GROUP BY subsystem ORDER BY COUNT(*) DESC",
        )?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }

    /// Get top N entries from a name+count query
    fn query_top_n(&self, sql: &str, limit: usize) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(sql)?;

        let rows = stmt
            .query_map(params![limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}

/// Collect SymbolRow from a prepared statement
fn collect_symbol_rows(
    stmt: &mut rusqlite::Statement,
    params: impl rusqlite::Params,
) -> Result<Vec<SymbolRow>> {
    let rows = stmt
        .query_map(params, |row| {
            Ok(SymbolRow {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                file_path: row.get(3)?,
                file_subsystem: row.get(4)?,
                line_start: row.get(5)?,
                line_end: row.get(6)?,
                is_exported: row.get(7)?,
                signature: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

// ============================================================================
// Subsystem detection
// ============================================================================

/// Linux kernel subsystem detection rules (ordered: more specific first)
const SUBSYSTEM_RULES: &[(&str, &str)] = &[
    ("drivers/usb/", "usb"),
    ("drivers/net/", "net-drivers"),
    ("drivers/gpu/", "gpu-drm"),
    ("drivers/block/", "block"),
    ("drivers/i2c/", "i2c"),
    ("drivers/spi/", "spi"),
    ("drivers/pci/", "pci"),
    ("drivers/mmc/", "mmc"),
    ("drivers/input/", "input"),
    ("drivers/tty/", "tty"),
    ("drivers/char/", "char"),
    ("drivers/platform/", "platform"),
    ("drivers/clk/", "clock"),
    ("drivers/regulator/", "regulator"),
    ("drivers/gpio/", "gpio"),
    ("drivers/pinctrl/", "pinctrl"),
    ("drivers/dma/", "dma"),
    ("drivers/iommu/", "iommu"),
    ("drivers/irqchip/", "irqchip"),
    ("drivers/watchdog/", "watchdog"),
    ("drivers/hwmon/", "hwmon"),
    ("drivers/thermal/", "thermal"),
    ("drivers/power/", "power"),
    ("drivers/mtd/", "mtd"),
    ("drivers/scsi/", "scsi"),
    ("drivers/", "drivers-other"),
    ("mm/", "memory"),
    ("kernel/sched/", "scheduler"),
    ("kernel/locking/", "locking"),
    ("kernel/irq/", "irq"),
    ("kernel/time/", "timekeeping"),
    ("kernel/", "kernel-core"),
    ("fs/ext4/", "ext4"),
    ("fs/btrfs/", "btrfs"),
    ("fs/xfs/", "xfs"),
    ("fs/nfs/", "nfs"),
    ("fs/proc/", "procfs"),
    ("fs/sysfs/", "sysfs"),
    ("fs/", "vfs"),
    ("net/core/", "net-core"),
    ("net/ipv4/", "ipv4"),
    ("net/ipv6/", "ipv6"),
    ("net/netfilter/", "netfilter"),
    ("net/wireless/", "wireless"),
    ("net/bluetooth/", "bluetooth"),
    ("net/", "net-other"),
    ("arch/arm/mach-", "arm-mach"),
    ("arch/arm/plat-", "arm-plat"),
    ("arch/arm/", "arm"),
    ("arch/arm64/", "arm64"),
    ("arch/x86/", "x86"),
    ("arch/riscv/", "riscv"),
    ("arch/", "arch-other"),
    ("sound/", "alsa"),
    ("crypto/", "crypto"),
    ("security/", "security"),
    ("lib/", "lib"),
    ("init/", "init"),
    ("ipc/", "ipc"),
    ("block/", "block-layer"),
];

/// Detect the kernel subsystem for a file path.
///
/// Uses path-segment-aware matching to avoid false positives
/// (e.g. `linux-kernel/` should not match `kernel/`).
pub fn detect_subsystem(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");

    // Split into path segments and rebuild candidate subpaths.
    // For "/Users/sky/linux-kernel/linux/arch/arm/mach-imx/foo.c"
    // we check "arch/arm/mach-imx/foo.c", "arm/mach-imx/foo.c", etc.
    let segments: Vec<&str> = normalized.split('/').collect();

    for start in 0..segments.len() {
        let subpath = segments[start..].join("/");
        for (prefix, subsystem) in SUBSYSTEM_RULES {
            if subpath.starts_with(prefix) {
                return Some((*subsystem).to_string());
            }
        }
    }

    None
}

/// Compute a fast hash of file content (for change detection)
pub fn hash_content(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Build a function signature string from parsed data
pub fn build_signature(
    name: &str,
    return_type: &str,
    params: &[flowsight_core::Parameter],
) -> String {
    let param_str: String = params
        .iter()
        .map(|p| format!("{} {}", p.type_name, p.name))
        .collect::<Vec<_>>()
        .join(", ");

    format!("{} {}({})", return_type, name, param_str)
}

/// Determine if a function is exported (heuristic for kernel code)
pub fn is_exported_fn(attrs: &[String], name: &str) -> bool {
    if attrs.iter().any(|a| a == "static") {
        return false;
    }
    if attrs.iter().any(|a| a.contains("EXPORT_SYMBOL")) {
        return true;
    }
    if attrs.iter().any(|a| a == "__init" || a == "__exit") {
        return true;
    }
    !name.starts_with('_')
}
