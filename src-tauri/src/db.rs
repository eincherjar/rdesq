use crate::crypto;
use crate::models::*;
use rusqlite::{params, Connection as SqlConn, Result as SqlResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<SqlConn>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> SqlResult<Self> {
        std::fs::create_dir_all(&app_dir).ok();
        let db_path = app_dir.join("rdesq.db");
        let conn = SqlConn::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn: Mutex::new(conn) };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT NOT NULL DEFAULT '',
                protocol TEXT NOT NULL DEFAULT 'ssh',
                auth_type TEXT NOT NULL DEFAULT 'password',
                password_enc TEXT NOT NULL DEFAULT '',
                private_key_path TEXT NOT NULL DEFAULT '',
                group_id TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                favorite INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS groups_t (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#9DD99A',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
        )?;
        // migrate existing databases that lack the favorite column
        conn.execute_batch("ALTER TABLE connections ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0")
            .ok();
        Ok(())
    }

    fn now() -> String {
        chrono::Utc::now().to_rfc3339()
    }

    fn row_to_entry(row: &rusqlite::Row) -> SqlResult<ConnEntry> {
        let pwd_enc: String = row.get(7)?;
        let pwd = if pwd_enc.is_empty() { None } else { crypto::decrypt(&pwd_enc).ok() };
        let tags_str: String = row.get(10)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        let gid: Option<String> = row.get(9)?;
        let gid = gid.filter(|s| !s.is_empty());
        Ok(ConnEntry {
            id: row.get(0)?,
            name: row.get(1)?,
            host: row.get(2)?,
            port: row.get(3)?,
            username: row.get(4)?,
            protocol: row.get(5)?,
            auth_type: row.get(6)?,
            password: pwd,
            private_key_path: {
                let v: String = row.get(8)?;
                if v.is_empty() { None } else { Some(v) }
            },
            group_id: gid,
            tags,
            sort_order: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            favorite: row.get(14)?,
        })
    }

    pub fn list_entries(&self) -> SqlResult<Vec<ConnEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, username, protocol, auth_type, password_enc, private_key_path, group_id, tags, sort_order, created_at, updated_at, favorite FROM connections ORDER BY sort_order, name",
        )?;
        let rows = stmt.query_map([], |row| Self::row_to_entry(row))?;
        rows.collect()
    }

    pub fn get_entry(&self, id: &str) -> SqlResult<Option<ConnEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, username, protocol, auth_type, password_enc, private_key_path, group_id, tags, sort_order, created_at, updated_at, favorite FROM connections WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| Self::row_to_entry(row))?;
        rows.next().transpose()
    }

    // ─── CONNECTIONS ─────────────────────────────

    pub fn list_connections(&self) -> SqlResult<Vec<ConnEntry>> {
        self.list_entries()
    }

    pub fn get_connection(&self, id: &str) -> SqlResult<Option<ConnEntry>> {
        self.get_entry(id)
    }

    pub fn create_connection(&self, input: ConnEntryInput) -> SqlResult<ConnEntry> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = Self::now();
        let pwd_enc = input.password.as_deref().map(crypto::encrypt).unwrap_or_default();
        let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();
        let gid = input.group_id.as_deref().unwrap_or("");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, username, protocol, auth_type, password_enc, private_key_path, group_id, tags, sort_order, created_at, updated_at, favorite) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
            params![
                id, input.name, input.host, input.port, input.username,
                input.protocol, input.auth_type, pwd_enc,
                input.private_key_path.as_deref().unwrap_or(""),
                gid, tags_json, 0, now, now, false
            ],
        )?;
        drop(conn);
        self.get_entry(&id).map(|o| o.unwrap())
    }

    pub fn update_connection(&self, id: &str, input: ConnEntryInput) -> SqlResult<Option<ConnEntry>> {
        let conn = self.conn.lock().unwrap();
        let now = Self::now();
        let pwd_enc = input.password.as_deref().map(crypto::encrypt).unwrap_or_default();
        let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();
        let gid = input.group_id.as_deref().unwrap_or("");
        let rows = conn.execute(
            "UPDATE connections SET name=?1, host=?2, port=?3, username=?4, protocol=?5, auth_type=?6, password_enc=?7, private_key_path=?8, group_id=?9, tags=?10, updated_at=?11 WHERE id=?12",
            params![
                input.name, input.host, input.port, input.username,
                input.protocol, input.auth_type, pwd_enc,
                input.private_key_path.as_deref().unwrap_or(""),
                gid, tags_json, now, id
            ],
        )?;
        if rows == 0 { return Ok(None); }
        drop(conn);
        self.get_entry(id)
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> SqlResult<Option<ConnEntry>> {
        let conn = self.conn.lock().unwrap();
        let now = Self::now();
        let rows = conn.execute(
            "UPDATE connections SET favorite=?1, updated_at=?2 WHERE id=?3",
            params![favorite, now, id],
        )?;
        if rows == 0 { return Ok(None); }
        drop(conn);
        self.get_entry(id)
    }

    pub fn delete_connection(&self, id: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM connections WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    pub fn duplicate_connection(&self, id: &str) -> SqlResult<Option<ConnEntry>> {
        let orig = self.get_entry(id)?;
        match orig {
            None => Ok(None),
            Some(c) => {
                let new_id = uuid::Uuid::new_v4().to_string();
                let now = Self::now();
                let new_name = format!("{} (copy)", c.name);
                let pwd_enc = c.password.as_deref().map(crypto::encrypt).unwrap_or_default();
                let tags_json = serde_json::to_string(&c.tags).unwrap_or_default();
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    "INSERT INTO connections (id, name, host, port, username, protocol, auth_type, password_enc, private_key_path, group_id, tags, sort_order, created_at, updated_at, favorite) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                    params![
                        new_id, new_name, c.host, c.port, c.username,
                        c.protocol, c.auth_type, pwd_enc,
                        c.private_key_path.as_deref().unwrap_or(""),
                        c.group_id.as_deref().unwrap_or(""),
                        tags_json, 0, now, now, false
                    ],
                )?;
                drop(conn);
                self.get_entry(&new_id)
            }
        }
    }

    // ─── GROUPS ───────────────────────────────────

    pub fn list_groups(&self) -> SqlResult<Vec<Group>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, sort_order, created_at, updated_at FROM groups_t ORDER BY sort_order, name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn create_group(&self, input: GroupInput) -> SqlResult<Group> {
        let conn = self.conn.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let now = Self::now();
        let color = input.color.unwrap_or_else(|| "#9DD99A".into());
        conn.execute(
            "INSERT INTO groups_t (id, name, color, sort_order, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, input.name, color, 0, now, now],
        )?;
        Ok(Group { id, name: input.name, color, sort_order: 0, created_at: now.clone(), updated_at: now })
    }

    pub fn update_group(&self, id: &str, input: GroupInput) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let now = Self::now();
        let rows = if let Some(color) = &input.color {
            conn.execute(
                "UPDATE groups_t SET name=?1, color=?2, updated_at=?3 WHERE id=?4",
                params![input.name, color, now, id],
            )?
        } else {
            conn.execute(
                "UPDATE groups_t SET name=?1, updated_at=?2 WHERE id=?3",
                params![input.name, now, id],
            )?
        };
        Ok(rows > 0)
    }

    pub fn delete_group(&self, id: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM groups_t WHERE id = ?1", params![id])?;
        if rows > 0 {
            conn.execute("UPDATE connections SET group_id = '' WHERE group_id = ?1", params![id])?;
        }
        Ok(rows > 0)
    }

    pub fn update_group_order(&self, orders: Vec<GroupOrder>) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        for o in &orders {
            conn.execute(
                "UPDATE groups_t SET sort_order = ?1 WHERE id = ?2",
                params![o.sort_order, o.id],
            )?;
        }
        Ok(())
    }

    // ─── SETTINGS ─────────────────────────────────

    pub fn get_settings(&self) -> Settings {
        let conn = self.conn.lock().unwrap();
        let read = |key: &str| -> Option<String> {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            ).ok()
        };
        Settings {
            lang: read("lang").unwrap_or_else(|| "pl".into()),
            theme: read("theme").unwrap_or_else(|| "dark".into()),
            ui_scale: read("ui_scale").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            start_with_system: read("start_with_system").map(|v| v == "true").unwrap_or(false),
            start_minimized: read("start_minimized").map(|v| v == "true").unwrap_or(false),
            close_to_tray: read("close_to_tray").map(|v| v == "true").unwrap_or(false),
        }
    }

    pub fn save_settings(&self, input: SettingsInput) {
        let conn = self.conn.lock().unwrap();
        let set = |key: &str, val: &str| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, val],
            ).ok();
        };
        if let Some(v) = &input.lang { set("lang", v); }
        if let Some(v) = &input.theme { set("theme", v); }
        if let Some(v) = input.ui_scale { set("ui_scale", &v.to_string()); }
        if let Some(v) = input.start_with_system { set("start_with_system", if v { "true" } else { "false" }); }
        if let Some(v) = input.start_minimized { set("start_minimized", if v { "true" } else { "false" }); }
        if let Some(v) = input.close_to_tray { set("close_to_tray", if v { "true" } else { "false" }); }
    }

    // ─── TAGS ─────────────────────────────────────

    pub fn list_tags(&self) -> SqlResult<Vec<TagInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT tags FROM connections WHERE tags != '[]'")?;
        let mut tag_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows {
            if let Ok(tags_str) = row {
                let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
                for t in tags {
                    *tag_counts.entry(t).or_insert(0) += 1;
                }
            }
        }
        let mut result: Vec<TagInfo> = tag_counts
            .into_iter()
            .map(|(name, count)| TagInfo { name, count })
            .collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    pub fn rename_tag(&self, old_name: &str, new_name: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, tags FROM connections WHERE tags != '[]'")?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        for (id, tags_str) in rows {
            let mut tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            let mut changed = false;
            for t in &mut tags {
                if t == old_name {
                    *t = new_name.to_string();
                    changed = true;
                }
            }
            if changed {
                let new_tags = serde_json::to_string(&tags).unwrap();
                conn.execute("UPDATE connections SET tags = ?1 WHERE id = ?2", params![new_tags, id])?;
            }
        }
        Ok(())
    }

    pub fn delete_tag(&self, name: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, tags FROM connections WHERE tags != '[]'")?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        for (id, tags_str) in rows {
            let mut tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            let len_before = tags.len();
            tags.retain(|t| t != name);
            if tags.len() != len_before {
                let new_tags = serde_json::to_string(&tags).unwrap();
                conn.execute("UPDATE connections SET tags = ?1 WHERE id = ?2", params![new_tags, id])?;
            }
        }
        Ok(())
    }

    // ─── EXPORT / IMPORT ──────────────────────────

    pub fn export_data(&self) -> SqlResult<ExportData> {
        let connections = self.list_entries()?;
        let groups = self.list_groups()?;
        Ok(ExportData { connections, groups })
    }

    fn get_group_by_id(&self, id: &str) -> SqlResult<Option<Group>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, sort_order, created_at, updated_at FROM groups_t WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.next().transpose()
    }

    fn get_group_by_name(&self, name: &str) -> SqlResult<Option<Group>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, color, sort_order, created_at, updated_at FROM groups_t WHERE name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        rows.next().transpose()
    }

    fn get_connection_by_host_port_protocol_username(
        &self,
        host: &str,
        port: u16,
        protocol: &str,
        username: &str,
    ) -> SqlResult<Option<ConnEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, host, port, username, protocol, auth_type, password_enc, private_key_path, group_id, tags, sort_order, created_at, updated_at, favorite FROM connections WHERE host = ?1 AND port = ?2 AND protocol = ?3 AND username = ?4",
        )?;
        let mut rows = stmt.query_map(params![host, port, protocol, username], |row| {
            Self::row_to_entry(row)
        })?;
        rows.next().transpose()
    }

    pub fn import_data(&self, data: ImportData) -> SqlResult<ImportResult> {
        let total = data.connections.len() + data.groups.len();
        let mut imported = 0;
        let mut group_id_map: HashMap<String, String> = HashMap::new();
        let mut group_name_map: HashMap<String, String> = HashMap::new();

        for g in &data.groups {
            let existing = self
                .get_group_by_id(&g.id)
                .ok()
                .flatten()
                .or_else(|| self.get_group_by_name(&g.name).ok().flatten());

            let result = if let Some(group) = existing {
                self.update_group(&group.id, GroupInput {
                    name: g.name.clone(),
                    color: Some(g.color.clone()),
                })
                .map(|_| group.id.clone())
            } else {
                self.create_group(GroupInput {
                    name: g.name.clone(),
                    color: Some(g.color.clone()),
                })
                .map(|group| group.id)
            };

            if let Ok(new_id) = result {
                group_id_map.insert(g.id.clone(), new_id.clone());
                group_name_map.insert(g.name.clone(), new_id);
                imported += 1;
            }
        }

        for c in &data.connections {
            let existing = self
                .get_entry(&c.id)
                .ok()
                .flatten()
                .or_else(|| {
                    self.get_connection_by_host_port_protocol_username(
                        &c.host, c.port, &c.protocol, &c.username,
                    )
                    .ok()
                    .flatten()
                });

            let resolved_group_id = c
                .group_id
                .as_ref()
                .and_then(|gid| group_id_map.get(gid).cloned())
                .or_else(|| {
                    c.group_id
                        .as_ref()
                        .and_then(|gid| group_name_map.get(gid).cloned())
                })
                .or_else(|| c.group_id.clone());

            let input = ConnEntryInput {
                name: c.name.clone(),
                host: c.host.clone(),
                port: c.port,
                username: c.username.clone(),
                protocol: c.protocol.clone(),
                auth_type: c.auth_type.clone(),
                password: c.password.clone(),
                private_key_path: c.private_key_path.clone(),
                group_id: resolved_group_id,
                tags: c.tags.clone(),
            };

            if let Some(conn) = existing {
                if self.update_connection(&conn.id, input).is_ok() {
                    if c.favorite && !conn.favorite {
                        self.set_favorite(&conn.id, true).ok();
                    }
                    imported += 1;
                }
            } else {
                if self.create_connection(input).is_ok() {
                    imported += 1;
                }
            }
        }

        Ok(ImportResult { imported, total })
    }
}
