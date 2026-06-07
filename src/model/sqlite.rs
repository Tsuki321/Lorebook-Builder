use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::{Connection, params};

use super::entry::WorldInfoEntry;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS entries (
    uid          INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    comment      TEXT NOT NULL DEFAULT '',
    content      TEXT NOT NULL DEFAULT '',
    priority     INTEGER NOT NULL DEFAULT 100,
    position     INTEGER NOT NULL DEFAULT 1,
    depth        INTEGER NOT NULL DEFAULT 4,
    probability  INTEGER NOT NULL DEFAULT 100,
    enabled      INTEGER NOT NULL DEFAULT 1,
    constant     INTEGER NOT NULL DEFAULT 0,
    selective    INTEGER NOT NULL DEFAULT 1,
    selective_logic INTEGER NOT NULL DEFAULT 0,
    disable      INTEGER NOT NULL DEFAULT 0,
    add_memo     INTEGER NOT NULL DEFAULT 1,
    exclude_recursion INTEGER NOT NULL DEFAULT 1,
    use_probability INTEGER NOT NULL DEFAULT 1,
    case_sensitive INTEGER NOT NULL DEFAULT 0,
    insertion_order INTEGER NOT NULL DEFAULT 100,
    pageid       INTEGER NOT NULL DEFAULT 0,
    revid        INTEGER NOT NULL DEFAULT 0,
    source_url   TEXT NOT NULL DEFAULT '',
    source_category TEXT NOT NULL DEFAULT '',
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entry_keys (
    uid   INTEGER NOT NULL,
    key   TEXT NOT NULL,
    kind  TEXT NOT NULL CHECK(kind IN ('primary','secondary')),
    PRIMARY KEY(uid, key, kind),
    FOREIGN KEY(uid) REFERENCES entries(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS crawl_runs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    wiki_url    TEXT NOT NULL,
    started_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    finished_at TEXT,
    total_pages INTEGER NOT NULL DEFAULT 0,
    ok_count    INTEGER NOT NULL DEFAULT 0,
    err_count   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_name ON entries(name);
CREATE INDEX IF NOT EXISTS idx_entry_keys_key ON entry_keys(key);
CREATE INDEX IF NOT EXISTS idx_entry_keys_uid ON entry_keys(uid);
"#;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)
            .with_context(|| format!("opening sqlite db at {}", path.display()))?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn upsert_entry(&self, entry: &WorldInfoEntry, pageid: u64, revid: u64,
                        source_url: &str, source_category: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            r#"INSERT INTO entries
                 (uid,name,comment,content,priority,position,depth,probability,
                  enabled,constant,selective,selective_logic,disable,add_memo,
                  exclude_recursion,use_probability,case_sensitive,insertion_order,
                  pageid,revid,source_url,source_category,updated_at)
               VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,CURRENT_TIMESTAMP)
               ON CONFLICT(uid) DO UPDATE SET
                 name=excluded.name,
                 comment=excluded.comment,
                 content=excluded.content,
                 priority=excluded.priority,
                 position=excluded.position,
                 depth=excluded.depth,
                 probability=excluded.probability,
                 enabled=excluded.enabled,
                 constant=excluded.constant,
                 selective=excluded.selective,
                 selective_logic=excluded.selective_logic,
                 disable=excluded.disable,
                 add_memo=excluded.add_memo,
                 exclude_recursion=excluded.exclude_recursion,
                 use_probability=excluded.use_probability,
                 case_sensitive=excluded.case_sensitive,
                 insertion_order=excluded.insertion_order,
                 pageid=excluded.pageid,
                 revid=excluded.revid,
                 source_url=excluded.source_url,
                 source_category=excluded.source_category,
                 updated_at=CURRENT_TIMESTAMP"#,
            params![
                entry.uid as i64,
                entry.name,
                entry.comment,
                entry.content,
                entry.priority as i64,
                entry.position as i64,
                entry.depth as i64,
                entry.probability as i64,
                entry.enabled as i64,
                entry.constant as i64,
                entry.selective as i64,
                entry.selective_logic as i64,
                entry.disable as i64,
                entry.add_memo as i64,
                entry.exclude_recursion as i64,
                entry.use_probability as i64,
                entry.case_sensitive as i64,
                entry.insertion_order as i64,
                pageid as i64,
                revid as i64,
                source_url,
                source_category,
            ],
        )?;
        tx.execute("DELETE FROM entry_keys WHERE uid = ?", params![entry.uid as i64])?;
        for k in &entry.keys {
            tx.execute(
                "INSERT OR IGNORE INTO entry_keys(uid,key,kind) VALUES (?,?,'primary')",
                params![entry.uid as i64, k],
            )?;
        }
        for k in &entry.secondary_keys {
            tx.execute(
                "INSERT OR IGNORE INTO entry_keys(uid,key,kind) VALUES (?,?,'secondary')",
                params![entry.uid as i64, k],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_entry(&self, uid: u64) -> Result<()> {
        self.conn.execute("DELETE FROM entries WHERE uid = ?", params![uid as i64])?;
        Ok(())
    }

    /// Wipe every entry (and its keys) from the library.
    /// Returns the number of rows deleted.
    pub fn clear_all(&self) -> Result<u64> {
        let n = self.conn.execute("DELETE FROM entries", [])? as u64;
        Ok(n)
    }

    pub fn max_uid(&self) -> Result<u64> {
        let v: Option<i64> = self.conn
            .query_row("SELECT MAX(uid) FROM entries", [], |r| r.get(0))?;
        Ok(v.unwrap_or(0).max(0) as u64)
    }

    pub fn count(&self) -> Result<u64> {
        let v: i64 = self.conn.query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;
        Ok(v.max(0) as u64)
    }

    pub fn list_all(&self) -> Result<Vec<WorldInfoEntry>> {
        self.list_all_with_keys()
    }

    /// Single-query equivalent of `list_all` + `hydrate_all`.
    /// Avoids the N+1 query pattern of calling `keys_for` per row.
    pub fn list_all_with_keys(&self) -> Result<Vec<WorldInfoEntry>> {
        // First fetch all keys grouped by uid in a single query.
        let mut keys_map: std::collections::HashMap<i64, (Vec<String>, Vec<String>)> =
            std::collections::HashMap::new();
        {
            let mut stmt = self.conn.prepare(
                "SELECT uid, key, kind FROM entry_keys ORDER BY uid, kind, key",
            )?;
            let rows = stmt.query_map([], |r| {
                let uid: i64 = r.get(0)?;
                let k: String = r.get(1)?;
                let kind: String = r.get(2)?;
                Ok((uid, k, kind))
            })?;
            for r in rows {
                let (uid, k, kind) = r?;
                let entry = keys_map.entry(uid).or_default();
                if kind == "secondary" {
                    entry.1.push(k);
                } else {
                    entry.0.push(k);
                }
            }
        }

        let mut stmt = self.conn.prepare(
            r#"SELECT uid,name,comment,content,priority,position,depth,probability,
                      enabled,constant,selective,selective_logic,disable,add_memo,
                      exclude_recursion,use_probability,case_sensitive,insertion_order
               FROM entries
               ORDER BY uid"#,
        )?;
        let rows = stmt.query_map([], |row| {
            let mut e = Self::row_to_entry(row)?;
            let uid = e.uid as i64;
            if let Some((primary, secondary)) = keys_map.remove(&uid) {
                e.keys = primary.clone();
                e.key = primary;
                e.secondary_keys = secondary.clone();
                e.keysecondary = secondary;
            }
            Ok(e)
        })?;
        let mut out = Vec::new();
        for r in rows { out.push(r?); }
        Ok(out)
    }

    pub fn search(&self, q: &str, limit: usize) -> Result<Vec<WorldInfoEntry>> {
        self.search_with_keys(q, limit)
    }

    /// Single-query search with keys attached.
    pub fn search_with_keys(&self, q: &str, limit: usize) -> Result<Vec<WorldInfoEntry>> {
        let pattern = format!("%{}%", q);
        // First, find matching uids via the cheap index lookup.
        let matching_uids: Vec<i64> = {
            let mut stmt = self.conn.prepare(
                r#"SELECT DISTINCT uid FROM entry_keys WHERE key LIKE ?1
                   UNION
                   SELECT uid FROM entries
                    WHERE name LIKE ?1 OR comment LIKE ?1 OR content LIKE ?1
                   ORDER BY uid LIMIT ?2"#,
            )?;
            let rows = stmt.query_map(params![pattern, limit as i64], |r| {
                let uid: i64 = r.get(0)?;
                Ok(uid)
            })?;
            let mut out = Vec::new();
            for r in rows { out.push(r?); }
            out
        };
        if matching_uids.is_empty() {
            return Ok(Vec::new());
        }

        // Then fetch full rows for those uids in one go.
        let placeholders: Vec<String> = (1..=matching_uids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            r#"SELECT uid,name,comment,content,priority,position,depth,probability,
                      enabled,constant,selective,selective_logic,disable,add_memo,
                      exclude_recursion,use_probability,case_sensitive,insertion_order
               FROM entries WHERE uid IN ({}) ORDER BY uid"#,
            placeholders.join(",")
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut params_vec: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(matching_uids.len());
        for u in &matching_uids { params_vec.push(u); }
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), Self::row_to_entry)?;

        // And keys for those uids in one go.
        let mut keys_map: std::collections::HashMap<i64, (Vec<String>, Vec<String>)> =
            std::collections::HashMap::new();
        {
            let placeholders_k: Vec<String> = (1..=matching_uids.len()).map(|i| format!("?{}", i)).collect();
            let sql_k = format!(
                "SELECT uid, key, kind FROM entry_keys WHERE uid IN ({}) ORDER BY uid, kind, key",
                placeholders_k.join(",")
            );
            let mut stmt_k = self.conn.prepare(&sql_k)?;
            let mut params_k: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(matching_uids.len());
            for u in &matching_uids { params_k.push(u); }
            let rows_k = stmt_k.query_map(rusqlite::params_from_iter(params_k), |r| {
                let uid: i64 = r.get(0)?;
                let k: String = r.get(1)?;
                let kind: String = r.get(2)?;
                Ok((uid, k, kind))
            })?;
            for r in rows_k {
                let (uid, k, kind) = r?;
                let entry = keys_map.entry(uid).or_default();
                if kind == "secondary" {
                    entry.1.push(k);
                } else {
                    entry.0.push(k);
                }
            }
        }

        let mut out = Vec::new();
        for r in rows {
            let mut e = r?;
            if let Some((primary, secondary)) = keys_map.remove(&(e.uid as i64)) {
                e.keys = primary.clone();
                e.key = primary;
                e.secondary_keys = secondary.clone();
                e.keysecondary = secondary;
            }
            out.push(e);
        }
        Ok(out)
    }

    fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorldInfoEntry> {
        let uid: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let comment: String = row.get(2)?;
        let content: String = row.get(3)?;
        let priority: i64 = row.get(4)?;
        let position: i64 = row.get(5)?;
        let depth: i64 = row.get(6)?;
        let probability: i64 = row.get(7)?;
        let enabled: i64 = row.get(8)?;
        let constant: i64 = row.get(9)?;
        let selective: i64 = row.get(10)?;
        let selective_logic: i64 = row.get(11)?;
        let disable: i64 = row.get(12)?;
        let add_memo: i64 = row.get(13)?;
        let exclude_recursion: i64 = row.get(14)?;
        let use_probability: i64 = row.get(15)?;
        let case_sensitive: i64 = row.get(16)?;
        let insertion_order: i64 = row.get(17)?;

        let uid_u = uid.max(0) as u64;
        let keys = Vec::new();
        let secondary = Vec::new();

        Ok(WorldInfoEntry {
            uid: uid_u,
            key: keys.clone(),
            keysecondary: secondary.clone(),
            comment,
            content,
            constant: constant != 0,
            selective: selective != 0,
            selective_logic: selective_logic.max(0).min(255) as u8,
            order: priority.max(0) as u32,
            position: position.max(0).min(255) as u8,
            disable: disable != 0,
            add_memo: add_memo != 0,
            exclude_recursion: exclude_recursion != 0,
            probability: probability.max(0).min(255) as u8,
            display_index: uid_u,
            use_probability: use_probability != 0,
            secondary_keys: secondary,
            keys,
            id: uid_u,
            priority: priority.max(0) as u32,
            insertion_order: insertion_order.max(0) as u32,
            enabled: enabled != 0,
            name,
            extensions: super::entry::EntryExtensions {
                depth: depth.max(0) as u32,
                weight: priority.max(0) as u32,
                add_memo: add_memo != 0,
                probability: probability.max(0).min(255) as u8,
                display_index: uid_u,
                selective_logic: selective_logic.max(0).min(255) as u8,
                use_probability: use_probability != 0,
                character_filter: None,
                exclude_recursion: exclude_recursion != 0,
            },
            case_sensitive: case_sensitive != 0,
            depth: depth.max(0) as u32,
            character_filter: None,
        })
    }

    /// Get the keys for an entry — call this after `list_all` / `search`.
    pub fn keys_for(&self, uid: u64) -> Result<(Vec<String>, Vec<String>)> {
        let mut primary = Vec::new();
        let mut secondary = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT key, kind FROM entry_keys WHERE uid = ?",
        )?;
        let rows = stmt.query_map(params![uid as i64], |r| {
            let k: String = r.get(0)?;
            let kind: String = r.get(1)?;
            Ok((k, kind))
        })?;
        for r in rows {
            let (k, kind) = r?;
            match kind.as_str() {
                "secondary" => secondary.push(k),
                _ => primary.push(k),
            }
        }
        Ok((primary, secondary))
    }

    pub fn hydrate(&self, mut entry: WorldInfoEntry) -> Result<WorldInfoEntry> {
        let (primary, secondary) = self.keys_for(entry.uid)?;
        entry.keys = primary.clone();
        entry.key = primary;
        entry.secondary_keys = secondary.clone();
        entry.keysecondary = secondary;
        Ok(entry)
    }

    pub fn hydrate_all(&self, entries: Vec<WorldInfoEntry>) -> Result<Vec<WorldInfoEntry>> {
        let mut out = Vec::with_capacity(entries.len());
        for e in entries { out.push(self.hydrate(e)?); }
        Ok(out)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings(key,value) VALUES (?,?) \
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let v = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?",
            params![key],
            |r| r.get::<_, String>(0),
        );
        match v {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
