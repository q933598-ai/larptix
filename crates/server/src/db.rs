use std::path::Path;
use std::sync::Mutex;

use larptrix_protocol::{attachment_url, avatar_url, AttachmentInfo, ChatMessage, UserInfo};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub avatar_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CryptoDeviceRow {
    pub user_id: String,
    pub bundle_json: String,
    pub encrypted_state: String,
}

#[derive(Debug, Clone)]
pub struct StoredFile {
    pub id: String,
    pub ext: String,
}

#[derive(Debug, Clone)]
pub struct GroupRow {
    pub id: String,
    pub name: String,
    pub member_ids: Vec<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn ping(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row("SELECT 1", [], |row| row.get::<_, i64>(0))
            .optional()?;
        Ok(())
    }

    pub fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
        created_at: i64,
    ) -> Result<UserRow, DbError> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().expect("db lock");
        match conn.execute(
            "INSERT INTO users (id, email, password_hash, display_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, email, password_hash, display_name, created_at],
        ) {
            Ok(_) => Ok(UserRow {
                id,
                email: email.to_string(),
                password_hash: password_hash.to_string(),
                display_name: display_name.to_string(),
                avatar_id: None,
            }),
            Err(err) if is_unique(&err) => Err(DbError::EmailTaken),
            Err(err) => Err(DbError::Sqlite(err)),
        }
    }

    pub fn create_key_user(
        &self,
        display_name: &str,
        access_key_hash: &str,
        created_at: i64,
    ) -> Result<UserRow, DbError> {
        let id = Uuid::new_v4().to_string();
        let internal_email = format!("{id}@key.larptrix.invalid");
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO users (id, email, password_hash, display_name, created_at, access_key_hash)
             VALUES (?1, ?2, '', ?3, ?4, ?5)",
            params![id, internal_email, display_name, created_at, access_key_hash],
        )
        .map_err(DbError::Sqlite)?;
        Ok(UserRow {
            id,
            email: internal_email,
            password_hash: String::new(),
            display_name: display_name.to_string(),
            avatar_id: None,
        })
    }

    pub fn user_by_access_key_hash(&self, key_hash: &str) -> rusqlite::Result<Option<UserRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT id, email, password_hash, display_name, avatar_id
             FROM users WHERE access_key_hash = ?1",
            [key_hash],
            map_user,
        )
        .optional()
    }

    pub fn crypto_device(&self, user_id: &str) -> rusqlite::Result<Option<CryptoDeviceRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT user_id, bundle_json, encrypted_state FROM crypto_devices WHERE user_id = ?1",
            [user_id],
            |row| {
                Ok(CryptoDeviceRow {
                    user_id: row.get(0)?,
                    bundle_json: row.get(1)?,
                    encrypted_state: row.get(2)?,
                })
            },
        )
        .optional()
    }

    pub fn save_crypto_device(
        &self,
        user_id: &str,
        bundle_json: &str,
        encrypted_state: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO crypto_devices (user_id, bundle_json, encrypted_state, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(user_id) DO UPDATE SET
                bundle_json = excluded.bundle_json,
                encrypted_state = excluded.encrypted_state,
                updated_at = excluded.updated_at",
            params![user_id, bundle_json, encrypted_state, crate::now_ms()],
        )?;
        Ok(())
    }

    pub fn activate_crypto_device(
        &self,
        user_id: &str,
        bundle_json: &str,
        encrypted_state: &str,
        _clear_history: bool,
    ) -> rusqlite::Result<Vec<StoredFile>> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction()?;
        let existing: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM crypto_devices WHERE user_id = ?1)",
            [user_id],
            |row| row.get(0),
        )?;
        if existing {
            tx.execute(
                "UPDATE crypto_devices SET bundle_json = ?1, encrypted_state = ?2, updated_at = ?3
                 WHERE user_id = ?4",
                params![bundle_json, encrypted_state, crate::now_ms(), user_id],
            )?;
            tx.commit()?;
            return Ok(Vec::new());
        }
        tx.execute(
            "INSERT INTO crypto_devices (user_id, bundle_json, encrypted_state, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![user_id, bundle_json, encrypted_state, crate::now_ms()],
        )?;
        tx.commit()?;
        Ok(Vec::new())
    }

    pub fn set_access_key_hash(&self, user_id: &str, key_hash: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE users SET access_key_hash = ?1 WHERE id = ?2",
            params![key_hash, user_id],
        )?;
        Ok(())
    }

    pub fn user_by_email(&self, email: &str) -> rusqlite::Result<Option<UserRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT id, email, password_hash, display_name, avatar_id
             FROM users WHERE email = ?1",
            [email],
            map_user,
        )
        .optional()
    }

    pub fn user_by_id(&self, id: &str) -> rusqlite::Result<Option<UserRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT id, email, password_hash, display_name, avatar_id
             FROM users WHERE id = ?1",
            [id],
            map_user,
        )
        .optional()
    }

    pub fn list_users(&self, online_ids: &[String]) -> rusqlite::Result<Vec<UserInfo>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT id, display_name, avatar_id, username,
                    EXISTS(SELECT 1 FROM crypto_devices WHERE crypto_devices.user_id = users.id)
                 FROM users ORDER BY display_name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let display_name: String = row.get(1)?;
            let avatar_id: Option<String> = row.get(2)?;
            let username: String = row.get(3)?;
            let e2e_enabled: bool = row.get(4)?;
            Ok(UserInfo {
                user_id: id.clone(),
                display_name,
                username,
                email: None,
                online: online_ids.iter().any(|online| online == &id),
                avatar_url: avatar_id.map(|_| avatar_url(&id)),
                is_group: false,
                e2e_enabled,
                group_member_ids: Vec::new(),
            })
        })?;
        rows.collect()
    }

    pub fn set_avatar(&self, user_id: &str, avatar_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE users SET avatar_id = ?1 WHERE id = ?2",
            params![avatar_id, user_id],
        )?;
        Ok(())
    }

    pub fn update_display_name(&self, user_id: &str, display_name: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE users SET display_name = ?1 WHERE id = ?2",
            params![display_name, user_id],
        )?;
        Ok(())
    }

    pub fn profile_fields(
        &self,
        user_id: &str,
    ) -> rusqlite::Result<Option<(String, String, String, Option<String>)>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT display_name, username, about, avatar_id FROM users WHERE id = ?1",
            [user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
    }

    pub fn update_profile(
        &self,
        user_id: &str,
        display_name: &str,
        username: &str,
        about: &str,
    ) -> Result<(), DbError> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE users SET display_name = ?1, username = ?2, about = ?3 WHERE id = ?4",
            params![display_name, username, about, user_id],
        )
        .map_err(|err| {
            if is_unique(&err) {
                DbError::BadRequest("username is already taken")
            } else {
                DbError::Sqlite(err)
            }
        })?;
        Ok(())
    }

    pub fn create_group(
        &self,
        creator_id: &str,
        name: &str,
        member_ids: &[String],
    ) -> Result<GroupRow, DbError> {
        let id = Uuid::new_v4().to_string();
        let conversation_id = group_conversation_key(&id);
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction().map_err(DbError::Sqlite)?;
        tx.execute(
            "INSERT INTO groups (id, name, created_by, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, creator_id, crate::now_ms()],
        )
        .map_err(DbError::Sqlite)?;
        let mut members = member_ids.to_vec();
        members.push(creator_id.to_string());
        members.sort();
        members.dedup();
        for member_id in &members {
            tx.execute(
                "INSERT INTO group_members (group_id, user_id) VALUES (?1, ?2)",
                params![id, member_id],
            )
            .map_err(|err| {
                if is_unique(&err) {
                    DbError::BadRequest("duplicate group member")
                } else {
                    DbError::Sqlite(err)
                }
            })?;
        }
        let (user_a, user_b) = ordered(&conversation_id, "group_members");
        tx.execute(
            "INSERT INTO conversations (id, user_a, user_b) VALUES (?1, ?2, ?3)",
            params![conversation_id, user_a, user_b],
        )
        .map_err(DbError::Sqlite)?;
        tx.commit().map_err(DbError::Sqlite)?;
        Ok(GroupRow {
            id,
            name: name.to_string(),
            member_ids: members,
        })
    }

    pub fn groups_for_user(&self, user_id: &str) -> rusqlite::Result<Vec<GroupRow>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT g.id, g.name FROM groups g
             JOIN group_members gm ON gm.group_id = g.id
             WHERE gm.user_id = ?1 ORDER BY g.name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([user_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut groups = Vec::new();
        for row in rows {
            let (id, name) = row?;
            let mut member_stmt = conn.prepare(
                "SELECT user_id FROM group_members WHERE group_id = ?1 ORDER BY user_id",
            )?;
            let member_ids = member_stmt
                .query_map([&id], |member| member.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            groups.push(GroupRow {
                id,
                name,
                member_ids,
            });
        }
        Ok(groups)
    }

    pub fn group(&self, group_id: &str) -> rusqlite::Result<Option<GroupRow>> {
        let conn = self.conn.lock().expect("db lock");
        let group = conn
            .query_row(
                "SELECT id, name FROM groups WHERE id = ?1",
                [group_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((id, name)) = group else {
            return Ok(None);
        };
        let mut stmt =
            conn.prepare("SELECT user_id FROM group_members WHERE group_id = ?1 ORDER BY user_id")?;
        let member_ids = stmt
            .query_map([&id], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok(Some(GroupRow {
            id,
            name,
            member_ids,
        }))
    }

    pub fn is_group_member(&self, group_id: &str, user_id: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = ?1 AND user_id = ?2)",
            params![group_id, user_id],
            |row| row.get(0),
        )
    }

    pub fn create_session(
        &self,
        token: &str,
        user_id: &str,
        expires_at: i64,
    ) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO sessions (token, user_id, expires_at) VALUES (?1, ?2, ?3)",
            params![token, user_id, expires_at],
        )?;
        Ok(())
    }

    pub fn user_by_session(&self, token: &str, now: i64) -> rusqlite::Result<Option<UserRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT u.id, u.email, u.password_hash, u.display_name, u.avatar_id
             FROM sessions s
             JOIN users u ON u.id = s.user_id
             WHERE s.token = ?1 AND s.expires_at > ?2",
            params![token, now],
            map_user,
        )
        .optional()
    }

    pub fn delete_session(&self, token: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute("DELETE FROM sessions WHERE token = ?1", [token])?;
        Ok(())
    }

    pub fn insert_attachment(
        &self,
        owner_id: &str,
        mime: &str,
        ext: &str,
        file_name: &str,
        byte_size: i64,
        created_at: i64,
    ) -> rusqlite::Result<String> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO attachments (id, owner_id, mime, ext, file_name, byte_size, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, owner_id, mime, ext, file_name, byte_size, created_at],
        )?;
        Ok(id)
    }

    pub fn attachment(&self, id: &str) -> rusqlite::Result<Option<AttachmentRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT id, owner_id, mime, ext, file_name, byte_size FROM attachments WHERE id = ?1",
            [id],
            |row| {
                Ok(AttachmentRow {
                    id: row.get(0)?,
                    owner_id: row.get(1)?,
                    mime: row.get(2)?,
                    ext: row.get(3)?,
                    file_name: row.get(4)?,
                    byte_size: row.get(5)?,
                })
            },
        )
        .optional()
    }

    pub fn can_view_attachment(
        &self,
        user_id: &str,
        attachment_id: &str,
    ) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("db lock");
        let owner: Option<String> = conn
            .query_row(
                "SELECT owner_id FROM attachments WHERE id = ?1",
                [attachment_id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(owner) = owner else {
            return Ok(false);
        };
        if owner == user_id {
            return Ok(true);
        }
        let used: i64 = conn.query_row(
            "SELECT COUNT(*)
             FROM messages m
             JOIN conversations c ON c.id = m.conversation_id
             WHERE m.attachment_id = ?1 AND (
                 c.user_a = ?2 OR c.user_b = ?2 OR EXISTS (
                     SELECT 1 FROM group_members gm
                     WHERE gm.user_id = ?2
                       AND m.conversation_id = 'group_' || gm.group_id
                 )
             )",
            params![attachment_id, user_id],
            |row| row.get(0),
        )?;
        Ok(used > 0)
    }

    pub fn insert_dm(
        &self,
        sender_id: &str,
        recipient_id: &str,
        body: &str,
        attachment_id: Option<&str>,
        created_at: i64,
    ) -> Result<ChatMessage, DbError> {
        if sender_id == recipient_id {
            return Err(DbError::BadRequest("cannot message yourself"));
        }
        let sender = self
            .user_by_id(sender_id)
            .map_err(DbError::Sqlite)?
            .ok_or(DbError::BadRequest("unknown sender"))?;
        let recipient = self
            .user_by_id(recipient_id)
            .map_err(DbError::Sqlite)?
            .ok_or(DbError::BadRequest("unknown user"))?;

        let conversation_id = conversation_key(sender_id, recipient_id);
        let (user_a, user_b) = ordered(sender_id, recipient_id);
        let message_id = Uuid::new_v4().to_string();
        let attachment = match attachment_id {
            Some(id) => Some(
                self.attachment(id)
                    .map_err(DbError::Sqlite)?
                    .ok_or(DbError::BadRequest("unknown attachment"))?,
            ),
            None => None,
        };
        if let Some(att) = &attachment {
            if att.owner_id != sender_id {
                return Err(DbError::BadRequest("attachment is not yours"));
            }
        }

        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT OR IGNORE INTO conversations (id, user_a, user_b) VALUES (?1, ?2, ?3)",
            params![conversation_id, user_a, user_b],
        )
        .map_err(DbError::Sqlite)?;
        conn.execute(
            "INSERT INTO messages (id, conversation_id, sender_id, body, attachment_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                message_id,
                conversation_id,
                sender_id,
                body,
                attachment.as_ref().map(|a| a.id.as_str()),
                created_at
            ],
        )
        .map_err(DbError::Sqlite)?;

        Ok(ChatMessage {
            id: message_id,
            sender_id: sender.id,
            sender_name: sender.display_name,
            recipient_id: recipient.id,
            body: body.to_string(),
            attachment: attachment.map(|a| AttachmentInfo {
                id: a.id.clone(),
                mime: a.mime,
                url: attachment_url(&a.id),
                name: a.file_name,
                size_bytes: a.byte_size,
            }),
            created_at,
        })
    }

    pub fn dm_history(
        &self,
        user_id: &str,
        peer_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, DbError> {
        if self.user_by_id(peer_id).map_err(DbError::Sqlite)?.is_none() {
            return Err(DbError::BadRequest("unknown user"));
        }
        let conversation_id = conversation_key(user_id, peer_id);
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
                "SELECT m.id, m.sender_id, u.display_name, m.body, m.attachment_id, a.mime, m.created_at,
                    a.file_name, a.byte_size
             FROM messages m
             JOIN users u ON u.id = m.sender_id
             LEFT JOIN attachments a ON a.id = m.attachment_id
             WHERE m.conversation_id = ?1
             ORDER BY m.created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![conversation_id, limit as i64], |row| {
            let sender_id: String = row.get(1)?;
            let attachment_id: Option<String> = row.get(4)?;
            let mime: Option<String> = row.get(5)?;
            Ok(ChatMessage {
                id: row.get(0)?,
                sender_id: sender_id.clone(),
                sender_name: row.get(2)?,
                recipient_id: if sender_id == user_id {
                    peer_id.to_string()
                } else {
                    user_id.to_string()
                },
                body: row.get(3)?,
                attachment: match (attachment_id, mime) {
                    (Some(id), Some(mime)) => Some(AttachmentInfo {
                        url: attachment_url(&id),
                        id,
                        mime,
                        name: row.get(7)?,
                        size_bytes: row.get(8)?,
                    }),
                    _ => None,
                },
                created_at: row.get(6)?,
            })
        })?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row.map_err(DbError::Sqlite)?);
        }
        messages.reverse();
        Ok(messages)
    }

    pub fn insert_group_dm(
        &self,
        sender_id: &str,
        group_id: &str,
        body: &str,
        attachment_id: Option<&str>,
        created_at: i64,
    ) -> Result<ChatMessage, DbError> {
        let group = self
            .group(group_id)
            .map_err(DbError::Sqlite)?
            .ok_or(DbError::BadRequest("unknown group"))?;
        if !group.member_ids.iter().any(|member| member == sender_id) {
            return Err(DbError::BadRequest("not a member of this group"));
        }
        let sender = self
            .user_by_id(sender_id)
            .map_err(DbError::Sqlite)?
            .ok_or(DbError::BadRequest("unknown sender"))?;
        let attachment = match attachment_id {
            Some(id) => Some(
                self.attachment(id)
                    .map_err(DbError::Sqlite)?
                    .ok_or(DbError::BadRequest("unknown attachment"))?,
            ),
            None => None,
        };
        if attachment
            .as_ref()
            .is_some_and(|file| file.owner_id != sender_id)
        {
            return Err(DbError::BadRequest("attachment is not yours"));
        }
        let message_id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO messages (id, conversation_id, sender_id, body, attachment_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                message_id,
                group_conversation_key(group_id),
                sender_id,
                body,
                attachment.as_ref().map(|file| file.id.as_str()),
                created_at
            ],
        )
        .map_err(DbError::Sqlite)?;
        Ok(ChatMessage {
            id: message_id,
            sender_id: sender.id,
            sender_name: sender.display_name,
            recipient_id: group.id,
            body: body.to_string(),
            attachment: attachment.map(|file| AttachmentInfo {
                id: file.id.clone(),
                mime: file.mime,
                url: attachment_url(&file.id),
                name: file.file_name,
                size_bytes: file.byte_size,
            }),
            created_at,
        })
    }

    pub fn group_history(
        &self,
        user_id: &str,
        group_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, DbError> {
        if !self
            .is_group_member(group_id, user_id)
            .map_err(DbError::Sqlite)?
        {
            return Err(DbError::BadRequest("not a member of this group"));
        }
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT m.id, m.sender_id, u.display_name, m.body, m.attachment_id, a.mime,
                    m.created_at, a.file_name, a.byte_size
             FROM messages m
             JOIN users u ON u.id = m.sender_id
             LEFT JOIN attachments a ON a.id = m.attachment_id
             WHERE m.conversation_id = ?1
             ORDER BY m.created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(
            params![group_conversation_key(group_id), limit as i64],
            |row| {
                let attachment_id: Option<String> = row.get(4)?;
                let mime: Option<String> = row.get(5)?;
                Ok(ChatMessage {
                    id: row.get(0)?,
                    sender_id: row.get(1)?,
                    sender_name: row.get(2)?,
                    recipient_id: group_id.to_string(),
                    body: row.get(3)?,
                    attachment: match (attachment_id, mime) {
                        (Some(id), Some(mime)) => Some(AttachmentInfo {
                            url: attachment_url(&id),
                            id,
                            mime,
                            name: row.get(7)?,
                            size_bytes: row.get(8)?,
                        }),
                        _ => None,
                    },
                    created_at: row.get(6)?,
                })
            },
        )?;
        let mut messages = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        messages.reverse();
        Ok(messages)
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentRow {
    pub id: String,
    pub owner_id: String,
    pub mime: String,
    pub ext: String,
    pub file_name: String,
    pub byte_size: i64,
}

#[derive(Debug)]
pub enum DbError {
    EmailTaken,
    BadRequest(&'static str),
    Sqlite(rusqlite::Error),
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Sqlite(err)
    }
}

fn map_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserRow> {
    Ok(UserRow {
        id: row.get(0)?,
        email: row.get(1)?,
        password_hash: row.get(2)?,
        display_name: row.get(3)?,
        avatar_id: row.get(4)?,
    })
}

fn ordered<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn conversation_key(a: &str, b: &str) -> String {
    let (low, high) = ordered(a, b);
    format!("{low}_{high}")
}

fn is_unique(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(info, _) if info.code == ErrorCode::ConstraintViolation
    )
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let has_messages: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'messages'",
        [],
        |row| row.get(0),
    )?;
    if has_messages > 0 {
        let has_conversation: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name = 'conversation_id'",
            [],
            |row| row.get(0),
        )?;
        if has_conversation == 0 {
            conn.execute_batch("ALTER TABLE messages RENAME TO messages_v01_room;")?;
        }
    }

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            display_name TEXT NOT NULL,
            avatar_id TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS crypto_devices (
            user_id TEXT PRIMARY KEY,
            bundle_json TEXT NOT NULL,
            encrypted_state TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            user_a TEXT NOT NULL,
            user_b TEXT NOT NULL,
            UNIQUE (user_a, user_b),
            CHECK (user_a < user_b)
        );
        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_by TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (created_by) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS group_members (
            group_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            PRIMARY KEY (group_id, user_id),
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS attachments (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            mime TEXT NOT NULL,
            ext TEXT NOT NULL,
            file_name TEXT NOT NULL DEFAULT 'attachment',
            byte_size INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (owner_id) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            attachment_id TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id),
            FOREIGN KEY (sender_id) REFERENCES users(id),
            FOREIGN KEY (attachment_id) REFERENCES attachments(id)
        );
        CREATE INDEX IF NOT EXISTS messages_conversation_created
            ON messages (conversation_id, created_at);
        "#,
    )?;
    let has_file_name: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('attachments') WHERE name = 'file_name'",
        [],
        |row| row.get(0),
    )?;
    if has_file_name == 0 {
        conn.execute(
            "ALTER TABLE attachments ADD COLUMN file_name TEXT NOT NULL DEFAULT 'attachment'",
            [],
        )?;
    }
    let has_access_key_hash: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'access_key_hash'",
        [],
        |row| row.get(0),
    )?;
    if has_access_key_hash == 0 {
        conn.execute("ALTER TABLE users ADD COLUMN access_key_hash TEXT", [])?;
    }
    for (column, definition) in [
        ("username", "TEXT NOT NULL DEFAULT ''"),
        ("about", "TEXT NOT NULL DEFAULT ''"),
    ] {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = ?1",
            [column],
            |row| row.get(0),
        )?;
        if exists == 0 {
            conn.execute_batch(&format!(
                "ALTER TABLE users ADD COLUMN {column} {definition};"
            ))?;
        }
    }
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS users_access_key_hash
         ON users(access_key_hash) WHERE access_key_hash IS NOT NULL",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS users_username
         ON users(username COLLATE NOCASE) WHERE username <> ''",
        [],
    )?;
    Ok(())
}

fn group_conversation_key(group_id: &str) -> String {
    format!("group_{group_id}")
}
#[cfg(test)]
mod tests {
    use super::Database;
    use std::path::Path;

    #[test]
    fn key_account_is_found_by_hash_only() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let user = db.create_key_user("Key User", "sha256-hash", 123).unwrap();
        let found = db.user_by_access_key_hash("sha256-hash").unwrap().unwrap();
        assert_eq!(found.id, user.id);
        assert!(found.email.ends_with("@key.larptrix.invalid"));
        assert!(db
            .user_by_access_key_hash("not-a-valid-key")
            .unwrap()
            .is_none());
    }

    #[test]
    fn legacy_account_can_be_assigned_an_access_key() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let user = db
            .create_user("legacy@example.test", "password-hash", "Legacy", 123)
            .unwrap();
        db.set_access_key_hash(&user.id, "new-key-hash").unwrap();
        let found = db.user_by_access_key_hash("new-key-hash").unwrap().unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.email, "legacy@example.test");
    }

    #[test]
    fn profile_fields_persist_and_usernames_are_unique_without_case_sensitivity() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "hash-a", 1).unwrap();
        let bob = db.create_key_user("Bob", "hash-b", 1).unwrap();
        db.update_profile(&alice.id, "Alice A", "alice_a", "Hello there")
            .unwrap();
        let (name, username, about, _) = db.profile_fields(&alice.id).unwrap().unwrap();
        assert_eq!(
            (name.as_str(), username.as_str(), about.as_str()),
            ("Alice A", "alice_a", "Hello there")
        );
        let directory = db.list_users(&[]).unwrap();
        assert_eq!(
            directory
                .iter()
                .find(|user| user.user_id == alice.id)
                .unwrap()
                .username,
            "alice_a"
        );
        assert!(db.update_profile(&bob.id, "Bob", "ALICE_A", "").is_err());
    }

    #[test]
    fn group_members_can_share_history_but_nonmembers_cannot_read_or_send() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "hash-a", 1).unwrap();
        let bob = db.create_key_user("Bob", "hash-b", 1).unwrap();
        let carol = db.create_key_user("Carol", "hash-c", 1).unwrap();
        let group = db
            .create_group(&alice.id, "Test group", std::slice::from_ref(&bob.id))
            .unwrap();
        assert_eq!(group.member_ids.len(), 2);
        assert_eq!(db.groups_for_user(&bob.id).unwrap().len(), 1);
        db.insert_group_dm(&alice.id, &group.id, "ciphertext-map", None, 1)
            .unwrap();
        assert_eq!(db.group_history(&bob.id, &group.id, 100).unwrap().len(), 1);
        assert!(db.group_history(&carol.id, &group.id, 100).is_err());
        assert!(db
            .insert_group_dm(&carol.id, &group.id, "plaintext", None, 2)
            .is_err());
    }

    #[test]
    fn first_e2e_activation_preserves_existing_history_and_attachments() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db
            .create_user("alice@example.test", "hash-a", "Alice", 1)
            .unwrap();
        let bob = db
            .create_user("bob@example.test", "hash-b", "Bob", 1)
            .unwrap();
        let avatar_id = db
            .insert_attachment(&alice.id, "image/png", "png", "avatar.png", 3, 1)
            .unwrap();
        db.set_avatar(&alice.id, &avatar_id).unwrap();
        let file_id = db
            .insert_attachment(
                &alice.id,
                "application/octet-stream",
                "txt",
                "old.txt",
                3,
                1,
            )
            .unwrap();
        db.insert_dm(&alice.id, &bob.id, "old plaintext", Some(&file_id), 1)
            .unwrap();
        db.insert_dm(&bob.id, &alice.id, "another old message", None, 2)
            .unwrap();
        let carol = db
            .create_user("carol@example.test", "hash-c", "Carol", 1)
            .unwrap();
        db.insert_dm(&bob.id, &carol.id, "preserved history", None, 3)
            .unwrap();

        let deleted = db
            .activate_crypto_device(&alice.id, "{}", "encrypted-state", true)
            .unwrap();
        assert!(deleted.is_empty());
        assert!(db.attachment(&avatar_id).unwrap().is_some());
        assert_eq!(db.dm_history(&alice.id, &bob.id, 100).unwrap().len(), 2);
        assert!(db.attachment(&file_id).unwrap().is_some());
        assert_eq!(db.dm_history(&bob.id, &carol.id, 100).unwrap().len(), 1);
        assert!(db.crypto_device(&alice.id).unwrap().is_some());
        let directory = db.list_users(&[]).unwrap();
        assert!(
            directory
                .iter()
                .find(|user| user.user_id == alice.id)
                .unwrap()
                .e2e_enabled
        );
        assert!(
            !directory
                .iter()
                .find(|user| user.user_id == bob.id)
                .unwrap()
                .e2e_enabled
        );
    }
}
