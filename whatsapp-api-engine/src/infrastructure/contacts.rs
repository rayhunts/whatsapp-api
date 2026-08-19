use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use whatsapp_api_errors::{AppError, AppResult};
use whatsapp_api_types::domain::contact::{
    Contact, ContactGroup, CreateContactGroupRequest, CreateContactRequest,
    UpdateContactGroupRequest, UpdateContactRequest,
};

const CONTACTS_DB_FILE: &str = "contacts.db";

/// SQLite-backed store for contacts and contact groups.
pub struct ContactStore {
    conn: Mutex<Connection>,
}

impl ContactStore {
    pub fn new(db_path: impl Into<String>) -> AppResult<Self> {
        let base: std::path::PathBuf = db_path.into().into();
        let dir = base.parent().unwrap_or_else(|| std::path::Path::new("."));
        let db_file = dir.join(CONTACTS_DB_FILE);
        let conn = Connection::open(&db_file)
            .map_err(|e| AppError::Storage(format!("failed to open contacts db: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                phone TEXT NOT NULL UNIQUE,
                email TEXT,
                notes TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Storage(format!("failed to create contacts table: {e}")))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contact_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                contact_ids TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Storage(format!("failed to create contact_groups table: {e}")))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn create_contact(&self, req: CreateContactRequest) -> AppResult<Contact> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_secs();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO contacts (id, name, phone, email, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![&id, &req.name, &req.phone, req.email, req.notes, now, now],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(Contact {
            id,
            name: req.name,
            phone: req.phone,
            email: req.email,
            notes: req.notes,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_contacts(&self) -> AppResult<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, phone, email, notes, created_at, updated_at
                 FROM contacts ORDER BY name COLLATE NOCASE",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], map_contact_row)
            .map_err(|e| AppError::Storage(e.to_string()))?;
        collect_rows(rows)
    }

    pub fn get_contact(&self, id: &str) -> AppResult<Contact> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, phone, email, notes, created_at, updated_at
                 FROM contacts WHERE id = ?1",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map([id], map_contact_row)
            .map_err(|e| AppError::Storage(e.to_string()))?;
        rows.next()
            .transpose()
            .map_err(|e| AppError::Storage(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("contact {id}")))
    }

    pub fn update_contact(&self, id: &str, req: UpdateContactRequest) -> AppResult<Contact> {
        let mut contact = self.get_contact(id)?;
        let now = now_secs();
        if let Some(name) = req.name {
            contact.name = name;
        }
        if let Some(phone) = req.phone {
            contact.phone = phone;
        }
        contact.email = req.email.or(contact.email);
        contact.notes = req.notes.or(contact.notes);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE contacts
             SET name = ?1, phone = ?2, email = ?3, notes = ?4, updated_at = ?5
             WHERE id = ?6",
            params![&contact.name, &contact.phone, contact.email, contact.notes, now, id],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        contact.updated_at = now;
        Ok(contact)
    }

    pub fn delete_contact(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM contacts WHERE id = ?1", [id])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("contact {id}")));
        }
        Ok(())
    }

    pub fn create_group(&self, req: CreateContactGroupRequest) -> AppResult<ContactGroup> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_secs();
        let contact_ids_json =
            serde_json::to_string(&req.contact_ids).map_err(|e| AppError::Storage(e.to_string()))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO contact_groups (id, name, contact_ids, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&id, &req.name, contact_ids_json, now, now],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(ContactGroup {
            id,
            name: req.name,
            contact_ids: req.contact_ids,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn list_groups(&self) -> AppResult<Vec<ContactGroup>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, contact_ids, created_at, updated_at
                 FROM contact_groups ORDER BY name COLLATE NOCASE",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], map_group_row)
            .map_err(|e| AppError::Storage(e.to_string()))?;
        collect_rows(rows)
    }

    pub fn get_group(&self, id: &str) -> AppResult<ContactGroup> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, contact_ids, created_at, updated_at
                 FROM contact_groups WHERE id = ?1",
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
        let mut rows = stmt
            .query_map([id], map_group_row)
            .map_err(|e| AppError::Storage(e.to_string()))?;
        rows.next()
            .transpose()
            .map_err(|e| AppError::Storage(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("contact group {id}")))
    }

    pub fn update_group(&self, id: &str, req: UpdateContactGroupRequest) -> AppResult<ContactGroup> {
        let mut group = self.get_group(id)?;
        let now = now_secs();
        if let Some(name) = req.name {
            group.name = name;
        }
        if let Some(contact_ids) = req.contact_ids {
            group.contact_ids = contact_ids;
        }
        let contact_ids_json =
            serde_json::to_string(&group.contact_ids).map_err(|e| AppError::Storage(e.to_string()))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE contact_groups
             SET name = ?1, contact_ids = ?2, updated_at = ?3
             WHERE id = ?4",
            params![&group.name, contact_ids_json, now, id],
        )
        .map_err(|e| AppError::Storage(e.to_string()))?;
        group.updated_at = now;
        Ok(group)
    }

    pub fn delete_group(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let affected = conn
            .execute("DELETE FROM contact_groups WHERE id = ?1", [id])
            .map_err(|e| AppError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(AppError::NotFound(format!("contact group {id}")));
        }
        Ok(())
    }

    /// Return the saved contact name for a raw WhatsApp JID, if one exists.
    pub fn contact_name_for_jid(&self, jid: &str) -> Option<String> {
        let digits: String = jid.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM contacts WHERE phone = ?1 OR phone = ?2 LIMIT 1")
            .ok()?;
        let name: Result<String, _> = stmt.query_row([&digits, &format!("+{digits}")], |row| row.get(0));
        name.ok()
    }

    /// Resolve contact group IDs and raw phone/JID entries into a deduplicated
    /// list of WhatsApp JIDs.
    pub fn resolve_broadcast_targets(
        &self,
        group_ids: &[String],
        to: &[String],
    ) -> AppResult<Vec<String>> {
        let mut targets: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        if !group_ids.is_empty() {
            let groups = self.list_groups()?;
            let contacts = self.list_contacts()?;
            let contact_by_id: std::collections::HashMap<_, _> =
                contacts.into_iter().map(|c| (c.id.clone(), c)).collect();

            for group in groups {
                if group_ids.contains(&group.id) {
                    for contact_id in &group.contact_ids {
                        if let Some(contact) = contact_by_id.get(contact_id)
                            && let Some(jid) =
                                whatsapp_api_types::domain::automation::phone_to_jid(&contact.phone)
                            && seen.insert(jid.clone())
                        {
                            targets.push(jid);
                        }
                    }
                }
            }
        }

        for entry in to {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let jid = if entry.contains('@') {
                entry.to_string()
            } else {
                whatsapp_api_types::domain::automation::phone_to_jid(entry)
                    .ok_or_else(|| AppError::InvalidInput(format!("invalid phone or jid: {entry}")))?
            };
            if seen.insert(jid.clone()) {
                targets.push(jid);
            }
        }

        Ok(targets)
    }
}

fn map_contact_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?,
        name: row.get(1)?,
        phone: row.get(2)?,
        email: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_group_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContactGroup> {
    let contact_ids_json: String = row.get(2)?;
    let contact_ids: Vec<String> = serde_json::from_str(&contact_ids_json).unwrap_or_default();
    Ok(ContactGroup {
        id: row.get(0)?,
        name: row.get(1)?,
        contact_ids,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn collect_rows<T>(rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>) -> AppResult<Vec<T>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|e| AppError::Storage(e.to_string()))?);
    }
    Ok(out)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
