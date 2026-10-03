use std::path::Path;
use std::sync::Mutex;

use larptrix_protocol::{attachment_url, avatar_url, AttachmentInfo, ChatMessage, UserInfo};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension, TransactionBehavior};
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
    pub device_id: String,
    pub user_id: String,
    pub bundle_json: String,
    pub encrypted_state: String,
    pub state_version: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct MatrixCryptoDeviceRow {
    pub device_id: String,
    pub user_id: String,
    pub device_keys_json: String,
    pub one_time_keys_json: String,
    pub fallback_keys_json: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct MatrixToDeviceRow {
    pub id: i64,
    pub recipient_user_id: String,
    pub recipient_device_id: String,
    pub sender_user_id: String,
    pub sender_device_id: String,
    pub event_type: String,
    pub txn_id: String,
    pub content_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct CryptoResyncRequestRow {
    pub id: i64,
    pub requester_user_id: String,
    pub target_user_id: String,
    pub message_id: String,
    pub device_id: String,
    pub body: String,
    pub attachment_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct CryptoResyncResponseRow {
    pub id: i64,
    pub recipient_user_id: String,
    pub sender_user_id: String,
    pub message_id: String,
    pub device_id: String,
    pub ciphertext: String,
    pub created_at: i64,
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
        username: &str,
        created_at: i64,
    ) -> Result<UserRow, DbError> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().expect("db lock");
        let username_taken: bool = conn.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM users WHERE username = ?1 COLLATE NOCASE AND username <> ''
                )",
                [username],
            |row| row.get(0),
        )?;
        if username_taken {
            return Err(DbError::UsernameTaken);
        }
        match conn.execute(
            "INSERT INTO users (id, email, password_hash, display_name, username, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, email, password_hash, display_name, username, created_at],
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
        username: &str,
        access_key_hash: &str,
        created_at: i64,
    ) -> Result<UserRow, DbError> {
        let id = Uuid::new_v4().to_string();
        let internal_email = format!("{id}@key.larptrix.invalid");
        let conn = self.conn.lock().expect("db lock");
        let username_taken: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM users WHERE username = ?1 COLLATE NOCASE AND username <> ''
                )",
                [username],
                |row| row.get(0),
            )?;
        if username_taken {
            return Err(DbError::UsernameTaken);
        }
        conn.execute(
            "INSERT INTO users (id, email, password_hash, display_name, username, created_at, access_key_hash)
             VALUES (?1, ?2, '', ?3, ?4, ?5, ?6)",
            params![id, internal_email, display_name, username, created_at, access_key_hash],
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

    pub fn crypto_device(&self, device_id: &str) -> rusqlite::Result<Option<CryptoDeviceRow>> {
        let conn = self.conn.lock().expect("db lock");

        conn.query_row(
            "SELECT device_id, user_id, bundle_json, encrypted_state,
                    state_version, created_at, updated_at
             FROM crypto_devices
             WHERE device_id = ?1",
            [device_id],
            |row| {
                Ok(CryptoDeviceRow {
                    device_id: row.get(0)?,
                    user_id: row.get(1)?,
                    bundle_json: row.get(2)?,
                    encrypted_state: row.get(3)?,
                    state_version: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .optional()
    }

    pub fn crypto_devices_for_user(&self, user_id: &str) -> rusqlite::Result<Vec<CryptoDeviceRow>> {
        let conn = self.conn.lock().expect("db lock");

        let mut stmt = conn.prepare(
            "SELECT device_id, user_id, bundle_json, encrypted_state,
                    state_version, created_at, updated_at
             FROM crypto_devices
             WHERE user_id = ?1
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([user_id], |row| {
            Ok(CryptoDeviceRow {
                device_id: row.get(0)?,
                user_id: row.get(1)?,
                bundle_json: row.get(2)?,
                encrypted_state: row.get(3)?,
                state_version: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    pub fn user_has_crypto_devices(&self, user_id: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("db lock");

        conn.query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM crypto_devices
                WHERE user_id = ?1
            )",
            [user_id],
            |row| row.get(0),
        )
    }

    pub fn create_crypto_device(
        &self,
        user_id: &str,
        device_id: &str,
        bundle_json: &str,
        encrypted_state: &str,
    ) -> rusqlite::Result<CryptoDeviceRow> {
        let now = crate::now_ms();
        let conn = self.conn.lock().expect("db lock");

        conn.execute(
            "INSERT INTO crypto_devices (
                device_id,
                user_id,
                bundle_json,
                encrypted_state,
                state_version,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
            params![device_id, user_id, bundle_json, encrypted_state, now],
        )?;

        Ok(CryptoDeviceRow {
            device_id: device_id.to_string(),
            user_id: user_id.to_string(),
            bundle_json: bundle_json.to_string(),
            encrypted_state: encrypted_state.to_string(),
            state_version: 1,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_crypto_device(
        &self,
        user_id: &str,
        device_id: &str,
        expected_version: i64,
        bundle_json: &str,
        encrypted_state: &str,
    ) -> rusqlite::Result<Option<CryptoDeviceRow>> {
        let now = crate::now_ms();
        let new_version = expected_version + 1;

        let conn = self.conn.lock().expect("db lock");

        let changed = conn.execute(
            "UPDATE crypto_devices
             SET bundle_json = ?1,
                 encrypted_state = ?2,
                 state_version = ?3,
                 updated_at = ?4
             WHERE device_id = ?5
               AND user_id = ?6
               AND state_version = ?7",
            params![
                bundle_json,
                encrypted_state,
                new_version,
                now,
                device_id,
                user_id,
                expected_version
            ],
        )?;

        if changed == 0 {
            return Ok(None);
        }

        let created_at = conn.query_row(
            "SELECT created_at
             FROM crypto_devices
             WHERE device_id = ?1",
            [device_id],
            |row| row.get(0),
        )?;

        Ok(Some(CryptoDeviceRow {
            device_id: device_id.to_string(),
            user_id: user_id.to_string(),
            bundle_json: bundle_json.to_string(),
            encrypted_state: encrypted_state.to_string(),
            state_version: new_version,
            created_at,
            updated_at: now,
        }))
    }

    pub fn claim_crypto_one_time_key(&self, device_id: &str) -> rusqlite::Result<Option<String>> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let bundle_json: Option<String> = tx
            .query_row(
                "SELECT bundle_json FROM crypto_devices WHERE device_id = ?1",
                [device_id],
                |row| row.get(0),
            )
            .optional()?;

        let Some(bundle_json) = bundle_json else {
            tx.commit()?;
            return Ok(None);
        };

        let mut bundle: serde_json::Value =
            serde_json::from_str(&bundle_json).map_err(|_| rusqlite::Error::InvalidQuery)?;

        let keys = bundle
            .get("one_time_keys")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        for key in keys {
            let Some(key_value) = key.get("key").and_then(serde_json::Value::as_str) else {
                continue;
            };

            let claimed: bool = tx.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM crypto_one_time_key_claims
                    WHERE device_id = ?1 AND key = ?2
                )",
                params![device_id, key_value],
                |row| row.get(0),
            )?;

            if claimed {
                continue;
            }

            tx.execute(
                "INSERT INTO crypto_one_time_key_claims (device_id, key, claimed_at)
                 VALUES (?1, ?2, ?3)",
                params![device_id, key_value, crate::now_ms()],
            )?;

            bundle["one_time_keys"] = serde_json::json!([key]);
            let claimed_bundle =
                serde_json::to_string(&bundle).map_err(|_| rusqlite::Error::InvalidQuery)?;

            tx.commit()?;
            return Ok(Some(claimed_bundle));
        }

        let fallback_keys = bundle
            .get("fallback_keys")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        if let Some(fallback) = fallback_keys.into_iter().find(|key| {
            key.get("key")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.is_empty())
        }) {
            // Fallback keys are reusable until the device publishes a replacement.
            bundle["one_time_keys"] = serde_json::json!([fallback]);
            let claimed_bundle =
                serde_json::to_string(&bundle).map_err(|_| rusqlite::Error::InvalidQuery)?;
            tx.commit()?;
            return Ok(Some(claimed_bundle));
        }

        tx.commit()?;
        Ok(None)
    }

    pub fn upsert_matrix_crypto_device(
        &self,
        user_id: &str,
        device_id: &str,
        device_keys_json: &str,
        one_time_keys_json: &str,
        fallback_keys_json: &str,
    ) -> rusqlite::Result<MatrixCryptoDeviceRow> {
        let now = crate::now_ms();
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction()?;

        let existing: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT created_at, one_time_keys_json, fallback_keys_json
                 FROM matrix_crypto_devices
                 WHERE device_id = ?1 AND user_id = ?2",
                params![device_id, user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        let created_at = existing.as_ref().map(|row| row.0).unwrap_or(now);

        let merge_maps = |existing_json: &str, incoming_json: &str, replace_if_nonempty: bool| {
            let mut merged = serde_json::Map::new();

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(existing_json) {
                if let Some(map) = value.as_object() {
                    for (key, value) in map {
                        merged.insert(key.clone(), value.clone());
                    }
                }
            }

            if let Ok(value) = serde_json::from_str::<serde_json::Value>(incoming_json) {
                if let Some(map) = value.as_object() {
                    if replace_if_nonempty && !map.is_empty() {
                        merged.clear();
                    }
                    for (key, value) in map {
                        merged.insert(key.clone(), value.clone());
                    }
                }
            }

            serde_json::Value::Object(merged).to_string()
        };

        let existing_otks = existing.as_ref().map(|row| row.1.as_str()).unwrap_or("{}");
        let existing_fallbacks = existing.as_ref().map(|row| row.2.as_str()).unwrap_or("{}");
        let merged_otks = merge_maps(existing_otks, one_time_keys_json, false);
        // A newly uploaded fallback-key set represents the current fallback
        // key set and therefore replaces the previous set when non-empty.
        let merged_fallbacks = merge_maps(existing_fallbacks, fallback_keys_json, true);

        tx.execute(
            "INSERT INTO matrix_crypto_devices (
                device_id, user_id, device_keys_json, one_time_keys_json,
                fallback_keys_json, created_at, updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(device_id) DO UPDATE SET
                user_id = excluded.user_id,
                device_keys_json = excluded.device_keys_json,
                one_time_keys_json = excluded.one_time_keys_json,
                fallback_keys_json = excluded.fallback_keys_json,
                updated_at = excluded.updated_at",
            params![
                device_id,
                user_id,
                device_keys_json,
                merged_otks,
                merged_fallbacks,
                created_at,
                now
            ],
        )?;

        tx.commit()?;

        Ok(MatrixCryptoDeviceRow {
            device_id: device_id.to_string(),
            user_id: user_id.to_string(),
            device_keys_json: device_keys_json.to_string(),
            one_time_keys_json: merged_otks,
            fallback_keys_json: merged_fallbacks,
            created_at,
            updated_at: now,
        })
    }

    pub fn matrix_devices_for_user(
        &self,
        user_id: &str,
    ) -> rusqlite::Result<Vec<MatrixCryptoDeviceRow>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT device_id, user_id, device_keys_json, one_time_keys_json,
                    fallback_keys_json, created_at, updated_at
             FROM matrix_crypto_devices
             WHERE user_id = ?1
             ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([user_id], |row| {
            Ok(MatrixCryptoDeviceRow {
                device_id: row.get(0)?,
                user_id: row.get(1)?,
                device_keys_json: row.get(2)?,
                one_time_keys_json: row.get(3)?,
                fallback_keys_json: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn matrix_device(
        &self,
        user_id: &str,
        device_id: &str,
    ) -> rusqlite::Result<Option<MatrixCryptoDeviceRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT device_id, user_id, device_keys_json, one_time_keys_json,
                    fallback_keys_json, created_at, updated_at
             FROM matrix_crypto_devices
             WHERE user_id = ?1 AND device_id = ?2",
            params![user_id, device_id],
            |row| {
                Ok(MatrixCryptoDeviceRow {
                    device_id: row.get(0)?,
                    user_id: row.get(1)?,
                    device_keys_json: row.get(2)?,
                    one_time_keys_json: row.get(3)?,
                    fallback_keys_json: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .optional()
    }

    pub fn matrix_one_time_key_count(&self, device_id: &str) -> rusqlite::Result<usize> {
        let conn = self.conn.lock().expect("db lock");

        let json: Option<String> = conn
            .query_row(
                "SELECT one_time_keys_json
                 FROM matrix_crypto_devices
                 WHERE device_id = ?1",
                [device_id],
                |row| row.get(0),
            )
            .optional()?;

        let Some(json) = json else {
            return Ok(0);
        };

        let value = serde_json::from_str::<serde_json::Value>(&json)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        let total = value.as_object().map(|map| map.len()).unwrap_or(0);

        let claimed: i64 = conn.query_row(
            "SELECT COUNT(*)
             FROM matrix_one_time_key_claims
             WHERE device_id = ?1",
            [device_id],
            |row| row.get(0),
        )?;

        Ok(total.saturating_sub(claimed as usize))
    }

    pub fn claim_matrix_one_time_key(
        &self,
        device_id: &str,
    ) -> rusqlite::Result<Option<(String, String)>> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let row: Option<(String, String)> = tx
            .query_row(
                "SELECT one_time_keys_json, fallback_keys_json
                 FROM matrix_crypto_devices WHERE device_id = ?1",
                [device_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let Some((otk_json, fallback_json)) = row else {
            tx.commit()?;
            return Ok(None);
        };

        let otks: serde_json::Value =
            serde_json::from_str(&otk_json).map_err(|_| rusqlite::Error::InvalidQuery)?;
        if let Some(map) = otks.as_object() {
            for (key_id, value) in map {
                let claimed: bool = tx.query_row(
                    "SELECT EXISTS(
                        SELECT 1 FROM matrix_one_time_key_claims
                        WHERE device_id = ?1 AND key_id = ?2
                    )",
                    params![device_id, key_id],
                    |row| row.get(0),
                )?;
                if claimed {
                    continue;
                }
                tx.execute(
                    "INSERT INTO matrix_one_time_key_claims(device_id, key_id, claimed_at)
                     VALUES (?1, ?2, ?3)",
                    params![device_id, key_id, crate::now_ms()],
                )?;
                let value_json =
                    serde_json::to_string(value).map_err(|_| rusqlite::Error::InvalidQuery)?;
                tx.commit()?;
                return Ok(Some((key_id.to_string(), value_json)));
            }
        }

        // Fallback keys are deliberately reusable until replaced.
        let fallbacks: serde_json::Value =
            serde_json::from_str(&fallback_json).map_err(|_| rusqlite::Error::InvalidQuery)?;
        if let Some(map) = fallbacks.as_object() {
            if let Some((key_id, value)) = map.iter().next() {
                let value_json =
                    serde_json::to_string(value).map_err(|_| rusqlite::Error::InvalidQuery)?;
                tx.commit()?;
                return Ok(Some((key_id.to_string(), value_json)));
            }
        }

        tx.commit()?;
        Ok(None)
    }

    pub fn enqueue_matrix_to_device(
        &self,
        recipient_user_id: &str,
        recipient_device_id: &str,
        sender_user_id: &str,
        sender_device_id: &str,
        event_type: &str,
        txn_id: &str,
        content_json: &str,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO matrix_to_device_events (
                recipient_user_id,
                recipient_device_id,
                sender_user_id,
                sender_device_id,
                event_type,
                txn_id,
                content_json,
                created_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(
                recipient_user_id,
                recipient_device_id,
                sender_user_id,
                sender_device_id,
                event_type,
                txn_id
             ) DO NOTHING",
            params![
                recipient_user_id,
                recipient_device_id,
                sender_user_id,
                sender_device_id,
                event_type,
                txn_id,
                content_json,
                crate::now_ms()
            ],
        )?;

        conn.query_row(
            "SELECT id
             FROM matrix_to_device_events
             WHERE recipient_user_id = ?1
               AND recipient_device_id = ?2
               AND sender_user_id = ?3
               AND sender_device_id = ?4
               AND event_type = ?5
               AND txn_id = ?6",
            params![
                recipient_user_id,
                recipient_device_id,
                sender_user_id,
                sender_device_id,
                event_type,
                txn_id
            ],
            |row| row.get(0),
        )
    }

    pub fn matrix_to_device_for_user(
        &self,
        user_id: &str,
    ) -> rusqlite::Result<Vec<MatrixToDeviceRow>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT id, recipient_user_id, recipient_device_id,
                    sender_user_id, sender_device_id, event_type,
                    txn_id, content_json, created_at
             FROM matrix_to_device_events
             WHERE recipient_user_id = ?1
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([user_id], |row| {
            Ok(MatrixToDeviceRow {
                id: row.get(0)?,
                recipient_user_id: row.get(1)?,
                recipient_device_id: row.get(2)?,
                sender_user_id: row.get(3)?,
                sender_device_id: row.get(4)?,
                event_type: row.get(5)?,
                txn_id: row.get(6)?,
                content_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        rows.collect()
    }

    pub fn ack_matrix_to_device(
        &self,
        user_id: &str,
        event_ids: &[i64],
    ) -> rusqlite::Result<usize> {
        let mut conn = self.conn.lock().expect("db lock");
        let tx = conn.transaction()?;
        let mut deleted = 0usize;
        for event_id in event_ids {
            deleted += tx.execute(
                "DELETE FROM matrix_to_device_events
                 WHERE id = ?1 AND recipient_user_id = ?2",
                params![event_id, user_id],
            )?;
        }
        tx.commit()?;
        Ok(deleted)
    }

    pub fn enqueue_crypto_resync_request(
        &self,
        requester_user_id: &str,
        target_user_id: &str,
        message_id: &str,
        device_id: &str,
        body: &str,
        attachment_id: Option<&str>,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO crypto_resync_requests (
                requester_user_id,
                target_user_id,
                message_id,
                device_id,
                body,
                attachment_id,
                created_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(
                requester_user_id,
                target_user_id,
                message_id,
                device_id
             ) DO NOTHING",
            params![
                requester_user_id,
                target_user_id,
                message_id,
                device_id,
                body,
                attachment_id,
                crate::now_ms()
            ],
        )?;

        conn.query_row(
            "SELECT id
             FROM crypto_resync_requests
             WHERE requester_user_id = ?1
               AND target_user_id = ?2
               AND message_id = ?3
               AND device_id = ?4",
            params![requester_user_id, target_user_id, message_id, device_id],
            |row| row.get(0),
        )
    }

    pub fn crypto_resync_requests_for_user(
        &self,
        target_user_id: &str,
    ) -> rusqlite::Result<Vec<CryptoResyncRequestRow>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT id, requester_user_id, target_user_id, message_id,
                    device_id, body, attachment_id, created_at
             FROM crypto_resync_requests
             WHERE target_user_id = ?1
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([target_user_id], |row| {
            Ok(CryptoResyncRequestRow {
                id: row.get(0)?,
                requester_user_id: row.get(1)?,
                target_user_id: row.get(2)?,
                message_id: row.get(3)?,
                device_id: row.get(4)?,
                body: row.get(5)?,
                attachment_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_crypto_resync_request(
        &self,
        requester_user_id: &str,
        target_user_id: &str,
        message_id: &str,
        device_id: &str,
    ) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("db lock");
        let deleted = conn.execute(
            "DELETE FROM crypto_resync_requests
             WHERE requester_user_id = ?1
               AND target_user_id = ?2
               AND message_id = ?3
               AND device_id = ?4",
            params![requester_user_id, target_user_id, message_id, device_id],
        )?;
        Ok(deleted > 0)
    }

    pub fn enqueue_crypto_resync_response(
        &self,
        recipient_user_id: &str,
        sender_user_id: &str,
        message_id: &str,
        device_id: &str,
        ciphertext: &str,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "INSERT INTO crypto_resync_responses (
                recipient_user_id,
                sender_user_id,
                message_id,
                device_id,
                ciphertext,
                created_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(
                recipient_user_id,
                sender_user_id,
                message_id,
                device_id
             ) DO UPDATE SET
                ciphertext = excluded.ciphertext",
            params![
                recipient_user_id,
                sender_user_id,
                message_id,
                device_id,
                ciphertext,
                crate::now_ms()
            ],
        )?;

        conn.query_row(
            "SELECT id
             FROM crypto_resync_responses
             WHERE recipient_user_id = ?1
               AND sender_user_id = ?2
               AND message_id = ?3
               AND device_id = ?4",
            params![recipient_user_id, sender_user_id, message_id, device_id],
            |row| row.get(0),
        )
    }

    pub fn crypto_resync_responses_for_user(
        &self,
        recipient_user_id: &str,
    ) -> rusqlite::Result<Vec<CryptoResyncResponseRow>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT id, recipient_user_id, sender_user_id, message_id,
                    device_id, ciphertext, created_at
             FROM crypto_resync_responses
             WHERE recipient_user_id = ?1
             ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([recipient_user_id], |row| {
            Ok(CryptoResyncResponseRow {
                id: row.get(0)?,
                recipient_user_id: row.get(1)?,
                sender_user_id: row.get(2)?,
                message_id: row.get(3)?,
                device_id: row.get(4)?,
                ciphertext: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn ack_crypto_resync_response(
        &self,
        recipient_user_id: &str,
        sender_user_id: &str,
        message_id: &str,
        device_id: &str,
    ) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("db lock");
        let deleted = conn.execute(
            "DELETE FROM crypto_resync_responses
             WHERE recipient_user_id = ?1
               AND sender_user_id = ?2
               AND message_id = ?3
               AND device_id = ?4",
            params![recipient_user_id, sender_user_id, message_id, device_id],
        )?;
        Ok(deleted > 0)
    }

    pub fn delete_crypto_device(&self, user_id: &str, device_id: &str) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().expect("db lock");

        let deleted = conn.execute(
            "DELETE FROM crypto_devices
             WHERE device_id = ?1
               AND user_id = ?2",
            params![device_id, user_id],
        )?;

        Ok(deleted != 0)
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

    pub fn user_by_username(&self, username: &str) -> rusqlite::Result<Option<UserRow>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT id, email, password_hash, display_name, avatar_id
             FROM users WHERE username = ?1 COLLATE NOCASE",
            [username],
            map_user,
        )
        .optional()
    }

    pub fn list_users(&self, online_ids: &[String]) -> rusqlite::Result<Vec<UserInfo>> {
        let conn = self.conn.lock().expect("db lock");
        let mut stmt = conn.prepare(
            "SELECT id, display_name, avatar_id, username,
                    EXISTS(SELECT 1 FROM crypto_devices WHERE crypto_devices.user_id = users.id),
                    activity
                 FROM users ORDER BY display_name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let display_name: String = row.get(1)?;
            let avatar_id: Option<String> = row.get(2)?;
            let username: String = row.get(3)?;
            let e2e_enabled: bool = row.get(4)?;
            let activity: String = row.get(5)?;
            let online = online_ids.iter().any(|online| online == &id);
            Ok(UserInfo {
                user_id: id.clone(),
                display_name,
                username,
                email: None,
                online,
                avatar_url: avatar_id.map(|_| avatar_url(&id)),
                activity: (online && !activity.is_empty()).then_some(activity),
                is_group: false,
                e2e_enabled,
                group_member_ids: Vec::new(),
            })
        })?;
        rows.collect()
    }

    pub fn set_activity(&self, user_id: &str, activity: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE users SET activity = ?1 WHERE id = ?2",
            params![activity, user_id],
        )?;
        Ok(())
    }

    pub fn activity(&self, user_id: &str) -> rusqlite::Result<String> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT activity FROM users WHERE id = ?1",
            [user_id],
            |row| row.get(0),
        )
    }

    pub fn set_avatar(&self, user_id: &str, avatar_id: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().expect("db lock");
        conn.execute(
            "UPDATE users SET avatar_id = ?1 WHERE id = ?2",
            params![avatar_id, user_id],
        )?;
        Ok(())
    }

    pub fn music_track(&self, user_id: &str) -> rusqlite::Result<Option<(String, String)>> {
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT a.id, a.file_name FROM users u
             JOIN attachments a ON a.id = u.music_attachment_id WHERE u.id = ?1",
            [user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
    }

    pub fn set_music_track(
        &self,
        user_id: &str,
        attachment_id: Option<&str>,
    ) -> rusqlite::Result<Option<(String, String)>> {
        let mut conn = self.conn.lock().expect("db lock");
        let transaction = conn.transaction()?;
        let previous: Option<String> = transaction.query_row(
            "SELECT music_attachment_id FROM users WHERE id = ?1",
            [user_id],
            |row| row.get(0),
        )?;
        transaction.execute(
            "UPDATE users SET music_attachment_id = ?1 WHERE id = ?2",
            params![attachment_id, user_id],
        )?;

        let removed_file = if previous
            .as_deref()
            .is_some_and(|old| Some(old) != attachment_id)
        {
            let old_id = previous.as_deref().unwrap();
            let ext: Option<String> = transaction
                .query_row(
                    "SELECT ext FROM attachments WHERE id = ?1",
                    [old_id],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(ext) = ext {
                let deleted = transaction.execute(
                    "DELETE FROM attachments WHERE id = ?1
                     AND NOT EXISTS (SELECT 1 FROM messages WHERE attachment_id = ?1)",
                    [old_id],
                )?;
                (deleted == 1).then(|| (old_id.to_string(), ext))
            } else {
                None
            }
        } else {
            None
        };
        transaction.commit()?;
        Ok(removed_file)
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

    pub fn dm_message_is_between(
        &self,
        message_id: &str,
        sender_id: &str,
        recipient_id: &str,
    ) -> rusqlite::Result<bool> {
        let conversation_id = conversation_key(sender_id, recipient_id);
        let conn = self.conn.lock().expect("db lock");
        conn.query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM messages
                WHERE id = ?1
                  AND conversation_id = ?2
                  AND sender_id = ?3
            )",
            params![message_id, conversation_id, sender_id],
            |row| row.get(0),
        )
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
    UsernameTaken,
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
            music_attachment_id TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );
        CREATE TABLE IF NOT EXISTS crypto_devices (
            device_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            bundle_json TEXT NOT NULL,
            encrypted_state TEXT NOT NULL,
            state_version INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_crypto_devices_user_id
            ON crypto_devices(user_id);
        CREATE TABLE IF NOT EXISTS crypto_one_time_key_claims (
            device_id TEXT NOT NULL,
            key TEXT NOT NULL,
            claimed_at INTEGER NOT NULL,
            PRIMARY KEY (device_id, key),
            FOREIGN KEY (device_id) REFERENCES crypto_devices(device_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_crypto_otk_claims_device
            ON crypto_one_time_key_claims(device_id);

        CREATE TABLE IF NOT EXISTS matrix_crypto_devices (
            device_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            device_keys_json TEXT NOT NULL,
            one_time_keys_json TEXT NOT NULL DEFAULT '{}',
            fallback_keys_json TEXT NOT NULL DEFAULT '{}',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_matrix_crypto_devices_user_id
            ON matrix_crypto_devices(user_id);

        CREATE TABLE IF NOT EXISTS matrix_one_time_key_claims (
            device_id TEXT NOT NULL,
            key_id TEXT NOT NULL,
            claimed_at INTEGER NOT NULL,
            PRIMARY KEY (device_id, key_id),
            FOREIGN KEY (device_id) REFERENCES matrix_crypto_devices(device_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_matrix_otk_claims_device
            ON matrix_one_time_key_claims(device_id);

        CREATE TABLE IF NOT EXISTS matrix_to_device_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient_user_id TEXT NOT NULL,
            recipient_device_id TEXT NOT NULL,
            sender_user_id TEXT NOT NULL,
            sender_device_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            txn_id TEXT NOT NULL,
            content_json TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (recipient_user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (sender_user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE (
                recipient_user_id,
                recipient_device_id,
                sender_user_id,
                sender_device_id,
                event_type,
                txn_id
            )
        );
        CREATE INDEX IF NOT EXISTS idx_matrix_to_device_recipient
            ON matrix_to_device_events(recipient_user_id, recipient_device_id, id);

        CREATE TABLE IF NOT EXISTS crypto_resync_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            requester_user_id TEXT NOT NULL,
            target_user_id TEXT NOT NULL,
            message_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            body TEXT NOT NULL,
            attachment_id TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (requester_user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (target_user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (attachment_id) REFERENCES attachments(id),
            UNIQUE (
                requester_user_id,
                target_user_id,
                message_id,
                device_id
            )
        );
        CREATE INDEX IF NOT EXISTS idx_crypto_resync_requests_target
            ON crypto_resync_requests(target_user_id, id);

        CREATE TABLE IF NOT EXISTS crypto_resync_responses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient_user_id TEXT NOT NULL,
            sender_user_id TEXT NOT NULL,
            message_id TEXT NOT NULL,
            device_id TEXT NOT NULL,
            ciphertext TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (recipient_user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (sender_user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE (
                recipient_user_id,
                sender_user_id,
                message_id,
                device_id
            )
        );
        CREATE INDEX IF NOT EXISTS idx_crypto_resync_responses_recipient
            ON crypto_resync_responses(recipient_user_id, id);

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
        ("music_attachment_id", "TEXT"),
        ("activity", "TEXT NOT NULL DEFAULT ''"),
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
    use uuid::Uuid;

    #[test]
    fn key_account_is_found_by_hash_only() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let user = db
            .create_key_user("Key User", "key_user", "sha256-hash", 123)
            .unwrap();
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
            .create_user(
                "legacy@example.test",
                "password-hash",
                "Legacy",
                "legacy",
                123,
            )
            .unwrap();
        db.set_access_key_hash(&user.id, "new-key-hash").unwrap();
        let found = db.user_by_access_key_hash("new-key-hash").unwrap().unwrap();
        assert_eq!(found.id, user.id);
        assert_eq!(found.email, "legacy@example.test");
    }

    #[test]
    fn profile_fields_persist_and_usernames_are_unique_without_case_sensitivity() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();
        let bob = db.create_key_user("Bob", "bob", "hash-b", 1).unwrap();
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
    fn profile_music_can_be_replaced_and_removed() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();
        let first = db
            .insert_attachment(&alice.id, "audio/mpeg", "mp3", "first.mp3", 4, 1)
            .unwrap();
        assert_eq!(db.set_music_track(&alice.id, Some(&first)).unwrap(), None);
        assert_eq!(
            db.music_track(&alice.id).unwrap(),
            Some((first.clone(), "first.mp3".to_string()))
        );

        let second = db
            .insert_attachment(&alice.id, "audio/ogg", "ogg", "second.ogg", 5, 2)
            .unwrap();
        assert_eq!(
            db.set_music_track(&alice.id, Some(&second)).unwrap(),
            Some((first.clone(), "mp3".to_string()))
        );
        assert!(db.attachment(&first).unwrap().is_none());
        assert_eq!(
            db.set_music_track(&alice.id, None).unwrap(),
            Some((second.clone(), "ogg".to_string()))
        );
        assert!(db.music_track(&alice.id).unwrap().is_none());
        assert!(db.attachment(&second).unwrap().is_none());
    }

    #[test]
    fn activity_is_only_published_for_online_users() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();
        db.set_activity(&alice.id, "Listening to song.mp3").unwrap();

        let online = db.list_users(std::slice::from_ref(&alice.id)).unwrap();
        assert_eq!(online[0].activity.as_deref(), Some("Listening to song.mp3"));

        let offline = db.list_users(&[]).unwrap();
        assert_eq!(offline[0].activity, None);
    }

    #[test]
    fn group_members_can_share_history_but_nonmembers_cannot_read_or_send() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();
        let bob = db.create_key_user("Bob", "bob", "hash-b", 1).unwrap();
        let carol = db.create_key_user("Carol", "carol", "hash-c", 1).unwrap();
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
    fn fallback_key_can_be_claimed_repeatedly_when_otks_are_exhausted() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();

        let device_id = Uuid::new_v4().to_string();
        let bundle = format!(
            r#"{{"version":2,"device_id":"{}","one_time_keys":[],"fallback_keys":[{{"key":"fallback","signature":"sig"}}]}}"#,
            device_id
        );
        db.create_crypto_device(&alice.id, &device_id, &bundle, "state")
            .unwrap();

        let first = db.claim_crypto_one_time_key(&device_id).unwrap().unwrap();
        let second = db.claim_crypto_one_time_key(&device_id).unwrap().unwrap();

        assert!(first.contains(r#""fallback""#));
        assert_eq!(first, second);
    }

    #[test]
    fn e2e_recovery_only_matches_the_original_dm_participants() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();
        let bob = db.create_key_user("Bob", "bob", "hash-b", 1).unwrap();
        let carol = db.create_key_user("Carol", "carol", "hash-c", 1).unwrap();

        let message = db
            .insert_dm(&alice.id, &bob.id, "encrypted", None, 1)
            .unwrap();

        assert!(db
            .dm_message_is_between(&message.id, &alice.id, &bob.id)
            .unwrap());
        assert!(!db
            .dm_message_is_between(&message.id, &alice.id, &carol.id)
            .unwrap());
        assert!(!db
            .dm_message_is_between(&message.id, &bob.id, &alice.id)
            .unwrap());
        assert!(!db
            .dm_message_is_between("missing", &alice.id, &bob.id)
            .unwrap());
    }

    #[test]
    fn matrix_one_time_keys_are_merged_and_claimed_once() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db.create_key_user("Alice", "alice", "hash-a", 1).unwrap();
        let bob = db.create_key_user("Bob", "bob", "hash-b", 1).unwrap();
        let device_id = Uuid::new_v4().to_string();

        db.upsert_matrix_crypto_device(
            &alice.id,
            &device_id,
            r#"{"user_id":"@alice:example.test","device_id":"DEV","keys":{}}"#,
            r#"{"signed_curve25519:a":{"key":"A"},"signed_curve25519:b":{"key":"B"}}"#,
            r#"{"signed_curve25519:f":{"key":"F"}}"#,
        )
        .unwrap();

        assert_eq!(db.matrix_one_time_key_count(&device_id).unwrap(), 2);

        let first = db.claim_matrix_one_time_key(&device_id).unwrap().unwrap();
        assert!(first.0 == "signed_curve25519:a" || first.0 == "signed_curve25519:b");
        assert_eq!(db.matrix_one_time_key_count(&device_id).unwrap(), 1);

        db.upsert_matrix_crypto_device(
            &alice.id,
            &device_id,
            r#"{"user_id":"@alice:example.test","device_id":"DEV","keys":{}}"#,
            r#"{"signed_curve25519:c":{"key":"C"}}"#,
            r#"{}"#,
        )
        .unwrap();

        assert_eq!(db.matrix_one_time_key_count(&device_id).unwrap(), 2);
        let second = db.claim_matrix_one_time_key(&device_id).unwrap().unwrap();
        assert_ne!(first.0, second.0);

        let event_a = db
            .enqueue_matrix_to_device(
                &bob.id,
                &device_id,
                &alice.id,
                &Uuid::new_v4().to_string(),
                "m.room_key",
                "txn-1",
                r#"{"ciphertext":"x"}"#,
            )
            .unwrap();
        let event_b = db
            .enqueue_matrix_to_device(
                &bob.id,
                &device_id,
                &alice.id,
                &Uuid::new_v4().to_string(),
                "m.room_key",
                "txn-1",
                r#"{"ciphertext":"x"}"#,
            )
            .unwrap();

        // Different sender device ids are different Matrix transactions.
        assert_ne!(event_a, event_b);

        let sender_device = Uuid::new_v4().to_string();
        let same_a = db
            .enqueue_matrix_to_device(
                &bob.id,
                &device_id,
                &alice.id,
                &sender_device,
                "m.room_key",
                "txn-2",
                r#"{"ciphertext":"y"}"#,
            )
            .unwrap();
        let same_b = db
            .enqueue_matrix_to_device(
                &bob.id,
                &device_id,
                &alice.id,
                &sender_device,
                "m.room_key",
                "txn-2",
                r#"{"ciphertext":"y"}"#,
            )
            .unwrap();
        assert_eq!(same_a, same_b);
    }

    #[test]
    fn first_e2e_activation_preserves_existing_history_and_attachments() {
        let db = Database::open(Path::new(":memory:")).unwrap();
        let alice = db
            .create_user("alice@example.test", "hash-a", "Alice", "alice", 1)
            .unwrap();
        let bob = db
            .create_user("bob@example.test", "hash-b", "Bob", "bob", 1)
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
            .create_user("carol@example.test", "hash-c", "Carol", "carol", 1)
            .unwrap();
        db.insert_dm(&bob.id, &carol.id, "preserved history", None, 3)
            .unwrap();

        let device_id_1 = Uuid::new_v4().to_string();

        let device_1 = db
            .create_crypto_device(
                &alice.id,
                &device_id_1,
                &format!(
                    r#"{{"version":2,"device_id":"{}","fingerprint":"alice-device-1"}}"#,
                    device_id_1
                ),
                "encrypted-state-1",
            )
            .unwrap();

        assert_eq!(device_1.device_id, device_id_1);
        assert_eq!(device_1.user_id, alice.id);
        assert_eq!(device_1.state_version, 1);

        assert!(db.attachment(&avatar_id).unwrap().is_some());
        assert_eq!(db.dm_history(&alice.id, &bob.id, 100).unwrap().len(), 2);
        assert!(db.attachment(&file_id).unwrap().is_some());
        assert_eq!(db.dm_history(&bob.id, &carol.id, 100).unwrap().len(), 1);

        let device_id_2 = Uuid::new_v4().to_string();

        let device_2 = db
            .create_crypto_device(
                &alice.id,
                &device_id_2,
                &format!(
                    r#"{{"version":2,"device_id":"{}","fingerprint":"alice-device-2"}}"#,
                    device_id_2
                ),
                "encrypted-state-2",
            )
            .unwrap();

        assert_eq!(device_2.state_version, 1);
        assert_ne!(device_1.device_id, device_2.device_id);

        let devices = db.crypto_devices_for_user(&alice.id).unwrap();

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].user_id, alice.id);
        assert_eq!(devices[1].user_id, alice.id);

        let updated = db
            .update_crypto_device(
                &alice.id,
                &device_id_1,
                1,
                &format!(
                    r#"{{"version":2,"device_id":"{}","fingerprint":"alice-device-1-updated"}}"#,
                    device_id_1
                ),
                "encrypted-state-1-updated",
            )
            .unwrap()
            .unwrap();

        assert_eq!(updated.state_version, 2);

        let stale_update = db
            .update_crypto_device(&alice.id, &device_id_1, 1, "{}", "stale-state")
            .unwrap();

        assert!(stale_update.is_none());

        assert!(db.crypto_device(&device_id_1).unwrap().is_some());
        assert!(db.user_has_crypto_devices(&alice.id).unwrap());

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
