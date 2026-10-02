use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use vodozemac::olm::{
    Account, AccountPickle, Message, OlmMessage, PreKeyMessage, Session, SessionConfig,
    SessionPickle,
};
use vodozemac::{base64_decode, Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature};
use wasm_bindgen::prelude::*;

const STATE_VERSION: u8 = 2;
const CIPHER_VERSION: u8 = 1;

const OTK_BATCH_SIZE: usize = 10;
const OTK_MINIMUM: usize = 3;
const MAX_SESSIONS_PER_PEER: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PublicOneTimeKey {
    key: String,
    signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicBundle {
    version: u8,
    device_id: String,
    curve_key: String,
    ed25519_key: String,
    identity_signature: String,
    one_time_keys: Vec<PublicOneTimeKey>,
    #[serde(default)]
    fallback_keys: Vec<PublicOneTimeKey>,
    fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CipherEnvelope {
    version: u8,
    message_type: String,
    ciphertext: String,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    version: u8,
    device_id: String,
    account_pickle: String,

    #[serde(default)]
    published_one_time_keys: Vec<PublicOneTimeKey>,

    #[serde(default)]
    published_fallback_keys: Vec<PublicOneTimeKey>,

    #[serde(deserialize_with = "deserialize_sessions")]
    sessions: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PersistedSessionList {
    Legacy(String),
    Current(Vec<String>),
}

fn deserialize_sessions<'de, D>(deserializer: D) -> Result<BTreeMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = BTreeMap::<String, PersistedSessionList>::deserialize(deserializer)?;

    Ok(stored
        .into_iter()
        .map(|(peer_id, sessions)| {
            let sessions = match sessions {
                PersistedSessionList::Legacy(session) => vec![session],
                PersistedSessionList::Current(sessions) => sessions,
            };

            (peer_id, sessions)
        })
        .collect())
}

fn restore_state_internal(
    recovery_key_base64: &str,
    state_json: &str,
) -> Result<CryptoDevice, String> {
    let recovery_key = decode_recovery_key_internal(recovery_key_base64)?;

    let state: PersistedState = serde_json::from_str(state_json)
        .map_err(|_| "invalid encrypted crypto state".to_string())?;

    if state.version != STATE_VERSION {
        return Err("unsupported encrypted crypto state version".to_string());
    }

    if state.device_id.is_empty() {
        return Err("crypto state is missing device id".to_string());
    }

    let account_pickle = AccountPickle::from_encrypted(&state.account_pickle, &recovery_key)
        .map_err(|_| "wrong recovery key or damaged crypto state".to_string())?;

    let account = Account::from(account_pickle);

    let sessions = state
        .sessions
        .into_iter()
        .map(|(peer_id, encrypted_sessions)| {
            let sessions = encrypted_sessions
                .into_iter()
                .map(|encrypted_session| {
                    let pickle = SessionPickle::from_encrypted(&encrypted_session, &recovery_key)
                        .map_err(|_| "could not restore encrypted session".to_string())?;

                    Ok(Session::from(pickle))
                })
                .collect::<Result<Vec<_>, String>>()?;

            Ok((peer_id, sessions))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;

    Ok(CryptoDevice {
        device_id: state.device_id,
        account,
        sessions,
        recovery_key,
        published_one_time_keys: state.published_one_time_keys,
    })
}

fn decode_recovery_key_internal(encoded: &str) -> Result<[u8; 32], String> {
    let bytes =
        base64_decode(encoded).map_err(|_| "recovery key must be unpadded base64".to_string())?;

    bytes
        .try_into()
        .map_err(|_| "recovery key must be exactly 32 bytes".to_string())
}

#[wasm_bindgen]
pub struct CryptoDevice {
    device_id: String,
    account: Account,
    sessions: BTreeMap<String, Vec<Session>>,
    recovery_key: [u8; 32],

    // Public metadata only.
    // Private one-time keys remain inside Account / AccountPickle.
    published_one_time_keys: Vec<PublicOneTimeKey>,
    published_fallback_keys: Vec<PublicOneTimeKey>,
}

#[wasm_bindgen]
impl CryptoDevice {
    #[wasm_bindgen(constructor)]
    pub fn create(recovery_key_base64: &str) -> Result<CryptoDevice, JsValue> {
        let recovery_key = decode_recovery_key(recovery_key_base64)?;
        let device_id = Uuid::new_v4().to_string();

        let mut account = Account::new();

        account.generate_one_time_keys(OTK_BATCH_SIZE);

        Ok(Self {
            device_id,
            account,
            sessions: BTreeMap::new(),
            recovery_key,
            published_one_time_keys: Vec::new(),
            published_fallback_keys: Vec::new(),
        })
    }

    #[wasm_bindgen(js_name = restore)]
    pub fn restore(recovery_key_base64: &str, state_json: &str) -> Result<CryptoDevice, JsValue> {
        restore_state_internal(recovery_key_base64, state_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn device_id(&self) -> String {
        self.device_id.clone()
    }

    pub fn fingerprint(&self) -> String {
        fingerprint(&self.account.curve25519_key(), &self.account.ed25519_key())
    }

    fn ensure_otk_pool(&mut self) -> Result<(), JsValue> {
        if self.account.stored_one_time_key_count() < OTK_MINIMUM {
            self.account.generate_one_time_keys(OTK_BATCH_SIZE);
        }

        if self.published_fallback_keys.is_empty() && self.account.fallback_key().is_empty() {
            self.account.generate_fallback_key();
        }

        Ok(())
    }

    pub fn public_bundle_json(&mut self) -> Result<String, JsValue> {
        self.ensure_otk_pool()?;

        let curve_key = self.account.curve25519_key();
        let ed25519_key = self.account.ed25519_key();

        let identity_signature = self.account.sign(curve_key.as_bytes()).to_base64();

        let mut one_time_keys = self.published_one_time_keys.clone();

        for (_, one_time_key) in self.account.one_time_keys() {
            let key = one_time_key.to_base64();

            if one_time_keys.iter().any(|existing| existing.key == key) {
                continue;
            }

            let signature = self.account.sign(one_time_key.as_bytes()).to_base64();

            one_time_keys.push(PublicOneTimeKey { key, signature });
        }

        let mut fallback_keys = self.published_fallback_keys.clone();

        for (_, fallback_key) in self.account.fallback_key() {
            let key = fallback_key.to_base64();

            if fallback_keys.iter().any(|existing| existing.key == key) {
                continue;
            }

            let signature = self.account.sign(fallback_key.as_bytes());

            fallback_keys.push(PublicOneTimeKey {
                key,
                signature: signature.to_base64(),
            });
        }

        serde_json::to_string(&PublicBundle {
            version: STATE_VERSION,
            device_id: self.device_id.clone(),
            curve_key: curve_key.to_base64(),
            ed25519_key: ed25519_key.to_base64(),
            identity_signature,
            one_time_keys,
            fallback_keys,
            fingerprint: fingerprint(&curve_key, &ed25519_key),
        })
        .map_err(|_| JsValue::from_str("could not encode public key bundle"))
    }

    pub fn mark_one_time_keys_as_published(&mut self) -> Result<(), JsValue> {
        let unpublished = self.account.one_time_keys();

        for (_, one_time_key) in unpublished {
            let key = one_time_key.to_base64();

            if self
                .published_one_time_keys
                .iter()
                .any(|existing| existing.key == key)
            {
                continue;
            }

            let signature = self.account.sign(one_time_key.as_bytes()).to_base64();

            self.published_one_time_keys
                .push(PublicOneTimeKey { key, signature });
        }

        let unpublished_fallback_keys = self.account.fallback_key();

        for (_, fallback_key) in unpublished_fallback_keys {
            let key = fallback_key.to_base64();

            if self
                .published_fallback_keys
                .iter()
                .any(|existing| existing.key == key)
            {
                continue;
            }

            let signature = self.account.sign(fallback_key.as_bytes());

            self.published_fallback_keys.push(PublicOneTimeKey {
                key,
                signature: signature.to_base64(),
            });
        }

        self.account.mark_keys_as_published();

        Ok(())
    }

    pub fn has_session(&self, peer_device_id: &str) -> bool {
        self.sessions
            .get(peer_device_id)
            .is_some_and(|sessions| !sessions.is_empty())
    }

    pub fn session_count(&self, peer_device_id: &str) -> usize {
        self.sessions.get(peer_device_id).map_or(0, Vec::len)
    }

    pub fn establish_session(
        &mut self,
        peer_device_id: &str,
        bundle_json: &str,
        expected_fingerprint: &str,
    ) -> Result<(), JsValue> {
        let bundle = parse_and_verify_bundle(bundle_json, expected_fingerprint)?;

        let identity_key = Curve25519PublicKey::from_base64(&bundle.curve_key)
            .map_err(|_| JsValue::from_str("invalid peer identity key"))?;

        let one_time_key = bundle
            .one_time_keys
            .first()
            .or_else(|| bundle.fallback_keys.first())
            .ok_or_else(|| JsValue::from_str("peer has no available one-time or fallback keys"))?;

        let one_time_key = Curve25519PublicKey::from_base64(&one_time_key.key)
            .map_err(|_| JsValue::from_str("invalid peer one-time key"))?;

        let session = self
            .account
            .create_outbound_session(SessionConfig::version_1(), identity_key, one_time_key)
            .map_err(|_| JsValue::from_str("could not establish encrypted session"))?;

        push_session(self.sessions.entry(peer_device_id.to_string()).or_default(), session);

        Ok(())
    }

    pub fn encrypt(&mut self, peer_device_id: &str, plaintext: &str) -> Result<String, JsValue> {
        let session = self
            .sessions
            .get_mut(peer_device_id)
            .and_then(|sessions| sessions.last_mut())
            .ok_or_else(|| JsValue::from_str("encrypted session is not established"))?;

        let message = session
            .encrypt(plaintext.as_bytes())
            .map_err(|_| JsValue::from_str("could not encrypt message"))?;

        let (message_type, ciphertext) = match message {
            OlmMessage::Normal(message) => ("message", message.to_base64()),
            OlmMessage::PreKey(message) => ("prekey", message.to_base64()),
        };

        serde_json::to_string(&CipherEnvelope {
            version: CIPHER_VERSION,
            message_type: message_type.to_string(),
            ciphertext,
        })
        .map_err(|_| JsValue::from_str("could not encode encrypted message"))
    }

    pub fn decrypt(
        &mut self,
        peer_device_id: &str,
        envelope_json: &str,
        sender_bundle_json: &str,
        expected_fingerprint: &str,
    ) -> Result<String, JsValue> {
        let envelope: CipherEnvelope = serde_json::from_str(envelope_json)
            .map_err(|_| JsValue::from_str("invalid encrypted message envelope"))?;

        if envelope.version != CIPHER_VERSION {
            return Err(JsValue::from_str("unsupported encrypted message version"));
        }

        match envelope.message_type.as_str() {
            "prekey" => self.decrypt_prekey(
                peer_device_id,
                &envelope.ciphertext,
                sender_bundle_json,
                expected_fingerprint,
            ),

            "message" => {
                let sessions = self
                    .sessions
                    .get_mut(peer_device_id)
                    .ok_or_else(|| JsValue::from_str("encrypted session is not established"))?;

                let plaintext =
                    decrypt_with_sessions(sessions, &envelope.ciphertext).ok_or_else(|| {
                        JsValue::from_str("could not decrypt message with any existing session")
                    })?;

                String::from_utf8(plaintext)
                    .map_err(|_| JsValue::from_str("decrypted message is not valid UTF-8"))
            }

            _ => Err(JsValue::from_str("unsupported encrypted message type")),
        }
    }

    fn decrypt_prekey(
        &mut self,
        peer_device_id: &str,
        ciphertext: &str,
        sender_bundle_json: &str,
        expected_fingerprint: &str,
    ) -> Result<String, JsValue> {
        let bundle = parse_and_verify_bundle(sender_bundle_json, expected_fingerprint)?;

        let sender_identity = Curve25519PublicKey::from_base64(&bundle.curve_key)
            .map_err(|_| JsValue::from_str("invalid sender identity key"))?;

        let pre_key = PreKeyMessage::from_base64(ciphertext)
            .map_err(|_| JsValue::from_str("invalid encrypted pre-key message"))?;

        if pre_key.identity_key() != sender_identity {
            return Err(JsValue::from_str(
                "sender identity does not match verified key",
            ));
        }

        if let Some(sessions) = self.sessions.get_mut(peer_device_id) {
            if let Some(plaintext) = decrypt_prekey_with_sessions(sessions, ciphertext) {
                return String::from_utf8(plaintext)
                    .map_err(|_| JsValue::from_str("decrypted message is not valid UTF-8"));
            }
        }

        let used_one_time_key = pre_key.one_time_key();

        let result = self
            .account
            .create_inbound_session(SessionConfig::version_1(), sender_identity, &pre_key)
            .map_err(|_| {
                JsValue::from_str(
                    "could not establish incoming encrypted session: one-time key unavailable",
                )
            })?;

        self.published_one_time_keys.retain(|entry| {
            Curve25519PublicKey::from_base64(&entry.key)
                .map(|key| key != used_one_time_key)
                .unwrap_or(true)
        });

        let plaintext = String::from_utf8(result.plaintext)
            .map_err(|_| JsValue::from_str("decrypted message is not valid UTF-8"))?;

        push_session(self.sessions.entry(peer_device_id.to_string()).or_default(), result.session);

        self.ensure_otk_pool()?;

        Ok(plaintext)
    }

    pub fn encrypted_state_json(&self) -> Result<String, JsValue> {
        let account_pickle = self.account.pickle().encrypt(&self.recovery_key);

        let sessions = self
            .sessions
            .iter()
            .map(|(peer_device_id, sessions)| {
                (
                    peer_device_id.clone(),
                    sessions
                        .iter()
                        .map(|session| session.pickle().encrypt(&self.recovery_key))
                        .collect(),
                )
            })
            .collect();

        serde_json::to_string(&PersistedState {
            version: STATE_VERSION,
            device_id: self.device_id.clone(),
            account_pickle,
            published_one_time_keys: self.published_one_time_keys.clone(),
            published_fallback_keys: self.published_fallback_keys.clone(),
            sessions,
        })
        .map_err(|_| JsValue::from_str("could not encrypt crypto state"))
    }
}

fn push_session(sessions: &mut Vec<Session>, session: Session) {
    sessions.push(session);
    if sessions.len() > MAX_SESSIONS_PER_PEER {
        let overflow = sessions.len() - MAX_SESSIONS_PER_PEER;
        sessions.drain(0..overflow);
    }
}

fn decrypt_with_sessions(sessions: &mut Vec<Session>, ciphertext: &str) -> Option<Vec<u8>> {
    let message = Message::from_base64(ciphertext).ok()?;

    for index in (0..sessions.len()).rev() {
        if let Ok(plaintext) = sessions[index].decrypt(&OlmMessage::Normal(message.clone())) {
            let session = sessions.remove(index);
            sessions.push(session);
            return Some(plaintext);
        }
    }

    None
}

fn decrypt_prekey_with_sessions(sessions: &mut Vec<Session>, ciphertext: &str) -> Option<Vec<u8>> {
    let message = PreKeyMessage::from_base64(ciphertext).ok()?;

    for index in (0..sessions.len()).rev() {
        if let Ok(plaintext) = sessions[index].decrypt(&OlmMessage::PreKey(message.clone())) {
            let session = sessions.remove(index);
            sessions.push(session);
            return Some(plaintext);
        }
    }

    None
}

fn decode_recovery_key(encoded: &str) -> Result<[u8; 32], JsValue> {
    let bytes = base64_decode(encoded)
        .map_err(|_| JsValue::from_str("recovery key must be unpadded base64"))?;

    bytes
        .try_into()
        .map_err(|_| JsValue::from_str("recovery key must be exactly 32 bytes"))
}

fn parse_and_verify_bundle(
    bundle_json: &str,
    expected_fingerprint: &str,
) -> Result<PublicBundle, JsValue> {
    let bundle: PublicBundle = serde_json::from_str(bundle_json)
        .map_err(|_| JsValue::from_str("invalid peer key bundle"))?;

    if bundle.version != STATE_VERSION {
        return Err(JsValue::from_str("unsupported peer key bundle version"));
    }

    if bundle.device_id.is_empty() {
        return Err(JsValue::from_str("peer bundle is missing device id"));
    }

    let curve_key = Curve25519PublicKey::from_base64(&bundle.curve_key)
        .map_err(|_| JsValue::from_str("invalid peer identity key"))?;

    let ed_key = Ed25519PublicKey::from_base64(&bundle.ed25519_key)
        .map_err(|_| JsValue::from_str("invalid peer signing key"))?;

    let actual_fingerprint = fingerprint(&curve_key, &ed_key);

    if actual_fingerprint != expected_fingerprint || bundle.fingerprint != actual_fingerprint {
        return Err(JsValue::from_str(
            "peer fingerprint does not match verified value",
        ));
    }

    let identity_signature = Ed25519Signature::from_base64(&bundle.identity_signature)
        .map_err(|_| JsValue::from_str("invalid identity signature"))?;

    ed_key
        .verify(curve_key.as_bytes(), &identity_signature)
        .map_err(|_| JsValue::from_str("identity signature verification failed"))?;

    if bundle.one_time_keys.is_empty() && bundle.fallback_keys.is_empty() {
        return Err(JsValue::from_str(
            "peer has no available one-time or fallback keys",
        ));
    }

    for one_time_key in bundle
        .one_time_keys
        .iter()
        .chain(bundle.fallback_keys.iter())
    {
        let key = Curve25519PublicKey::from_base64(&one_time_key.key)
            .map_err(|_| JsValue::from_str("invalid one-time or fallback key"))?;

        let signature = Ed25519Signature::from_base64(&one_time_key.signature)
            .map_err(|_| JsValue::from_str("invalid one-time or fallback key signature"))?;

        ed_key
            .verify(key.as_bytes(), &signature)
            .map_err(|_| {
                JsValue::from_str("one-time or fallback key signature verification failed")
            })?;
    }

    Ok(bundle)
}

fn fingerprint(curve_key: &Curve25519PublicKey, ed_key: &Ed25519PublicKey) -> String {
    let mut hasher = Sha256::new();

    hasher.update(curve_key.as_bytes());
    hasher.update(ed_key.as_bytes());

    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::restore_state_internal;
    use std::collections::BTreeMap;

    use super::{
        decrypt_prekey_with_sessions, decrypt_with_sessions, deserialize_sessions, CryptoDevice,
        PublicBundle, STATE_VERSION,
    };

    use vodozemac::base64_encode;
    use vodozemac::olm::{Account, OlmMessage, SessionConfig};

    #[test]
    fn olm_encrypt_decrypt_and_recovery_round_trip() {
        let alice = Account::new();

        let mut bob = Account::new();
        bob.generate_one_time_keys(1);

        let bob_one_time_key = bob.one_time_keys().into_values().next().unwrap();

        let mut alice_session = alice
            .create_outbound_session(
                SessionConfig::version_1(),
                bob.curve25519_key(),
                bob_one_time_key,
            )
            .unwrap();

        let first = alice_session.encrypt("hello over olm").unwrap();

        let OlmMessage::PreKey(pre_key) = first else {
            panic!("the first Olm message must establish a session");
        };

        let inbound = bob
            .create_inbound_session(SessionConfig::version_1(), alice.curve25519_key(), &pre_key)
            .unwrap();

        assert_eq!(
            String::from_utf8(inbound.plaintext).unwrap(),
            "hello over olm"
        );

        let mut bob_session = inbound.session;

        let reply = bob_session.encrypt("reply over olm").unwrap();

        let OlmMessage::Normal(reply) = reply else {
            panic!("reply must use the established Olm session");
        };

        assert_eq!(
            String::from_utf8(alice_session.decrypt(&OlmMessage::Normal(reply)).unwrap()).unwrap(),
            "reply over olm"
        );

        let recovery_key = [7u8; 32];

        let encrypted_account = alice.pickle().encrypt(&recovery_key);

        let restored = Account::from(
            vodozemac::olm::AccountPickle::from_encrypted(&encrypted_account, &recovery_key)
                .unwrap(),
        );

        assert_eq!(restored.curve25519_key(), alice.curve25519_key());
    }

    #[test]
    fn device_has_unique_device_id() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let alice = CryptoDevice::create(&recovery_key).unwrap();
        let bob = CryptoDevice::create(&recovery_key).unwrap();

        assert_ne!(alice.device_id, bob.device_id);
        assert_eq!(alice.device_id.len(), 36);
    }

    #[test]
    fn new_device_has_otk_pool() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let mut device = CryptoDevice::create(&recovery_key).unwrap();

        let bundle = device.public_bundle_json().unwrap();
        let bundle: PublicBundle = serde_json::from_str(&bundle).unwrap();

        assert_eq!(bundle.version, STATE_VERSION);
        assert_eq!(bundle.device_id, device.device_id);
        assert_eq!(bundle.one_time_keys.len(), 10);

        for one_time_key in &bundle.one_time_keys {
            assert!(!one_time_key.key.is_empty());
            assert!(!one_time_key.signature.is_empty());
        }

        assert_eq!(device.account.stored_one_time_key_count(), 10);
    }

    #[test]
    fn fallback_key_is_published_and_persists_across_restore() {
        let recovery_key = base64_encode(&[17u8; 32]);
        let mut device = CryptoDevice::create(&recovery_key).unwrap();

        let bundle = device.public_bundle_json().unwrap();
        let bundle: PublicBundle = serde_json::from_str(&bundle).unwrap();

        assert_eq!(bundle.fallback_keys.len(), 1);
        assert!(!bundle.fallback_keys[0].key.is_empty());
        assert!(!bundle.fallback_keys[0].signature.is_empty());

        device.mark_one_time_keys_as_published().unwrap();

        let persisted = device.encrypted_state_json().unwrap();
        let restored = CryptoDevice::restore(&recovery_key, &persisted).unwrap();
        let restored_bundle: PublicBundle =
            serde_json::from_str(&restored.public_bundle_json().unwrap()).unwrap();

        assert_eq!(restored_bundle.fallback_keys, bundle.fallback_keys);
    }

    #[test]
    fn publishing_otks_preserves_private_pool() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let mut device = CryptoDevice::create(&recovery_key).unwrap();

        let before = device.public_bundle_json().unwrap();
        let before: PublicBundle = serde_json::from_str(&before).unwrap();

        assert_eq!(before.one_time_keys.len(), 10);

        device.mark_one_time_keys_as_published().unwrap();

        assert_eq!(device.account.stored_one_time_key_count(), 10);

        let after = device.public_bundle_json().unwrap();
        let after: PublicBundle = serde_json::from_str(&after).unwrap();

        assert_eq!(after.one_time_keys.len(), 10);
        assert_eq!(after.one_time_keys, device.published_one_time_keys);
    }

    #[test]
    fn state_round_trip_preserves_device_account_and_public_otks() {
        let recovery_key = base64_encode(&[9u8; 32]);

        let mut device = CryptoDevice::create(&recovery_key).unwrap();

        device.mark_one_time_keys_as_published().unwrap();

        let device_id = device.device_id.clone();
        let fingerprint = device.fingerprint();
        let public_otks = device.published_one_time_keys.clone();

        let state = device.encrypted_state_json().unwrap();

        let restored = CryptoDevice::restore(&recovery_key, &state).unwrap();

        assert_eq!(restored.device_id, device_id);
        assert_eq!(restored.fingerprint(), fingerprint);
        assert_eq!(restored.published_one_time_keys, public_otks);
        assert_eq!(restored.account.stored_one_time_key_count(), 10);
    }

    #[test]
    fn multiple_sessions_are_retained_and_capped() {
        let recovery_key = base64_encode(&[11u8; 32]);
        let mut alice = CryptoDevice::create(&recovery_key).unwrap();
        let mut bob = CryptoDevice::create(&recovery_key).unwrap();

        let bundle = bob.public_bundle_json().unwrap();
        let bob_bundle: PublicBundle = serde_json::from_str(&bundle).unwrap();

        for _ in 0..5 {
            alice
                .establish_session(
                    &bob_bundle.device_id,
                    &bundle,
                    &bob_bundle.fingerprint,
                )
                .unwrap();
        }

        assert_eq!(alice.session_count(&bob_bundle.device_id), 4);
    }

    #[test]
    fn encrypted_state_contains_version_and_device_id() {
        let recovery_key = base64_encode(&[3u8; 32]);

        let device = CryptoDevice::create(&recovery_key).unwrap();

        let state = device.encrypted_state_json().unwrap();
        let json: serde_json::Value = serde_json::from_str(&state).unwrap();

        assert_eq!(json["version"], STATE_VERSION);
        assert_eq!(json["device_id"], device.device_id);
        assert!(json.get("one_time_key").is_none());
        assert!(json.get("one_time_signature").is_none());
    }

    #[test]
    fn prekey_session_round_trip_works() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let mut alice = CryptoDevice::create(&recovery_key).unwrap();
        let mut bob = CryptoDevice::create(&recovery_key).unwrap();

        let alice_bundle = alice.public_bundle_json().unwrap();
        let alice_fingerprint = alice.fingerprint();

        let bob_bundle = bob.public_bundle_json().unwrap();
        let bob_fingerprint = bob.fingerprint();

        alice
            .establish_session(&bob.device_id, &bob_bundle, &bob_fingerprint)
            .unwrap();

        let encrypted = alice.encrypt(&bob.device_id, "hello from alice").unwrap();

        assert_eq!(
            bob.decrypt(
                &alice.device_id,
                &encrypted,
                &alice_bundle,
                &alice_fingerprint,
            )
            .unwrap(),
            "hello from alice"
        );
    }

    #[test]
    fn fallback_only_bundle_is_accepted() {
        let account = Account::new();
        let fallback = {
            let mut account = account;
            account.generate_fallback_key();
            let key = *account.fallback_key().values().next().unwrap();
            let signature = account.sign(key.as_bytes()).to_base64();
            (account, key.to_base64(), signature)
        };

        let (account, fallback_key, signature) = fallback;
        let bundle = PublicBundle {
            version: STATE_VERSION,
            device_id: "fallback-device".to_string(),
            curve_key: account.curve25519_key().to_base64(),
            ed25519_key: account.ed25519_key().to_base64(),
            identity_signature: account
                .sign(account.curve25519_key().as_bytes())
                .to_base64(),
            one_time_keys: Vec::new(),
            fallback_keys: vec![PublicOneTimeKey {
                key: fallback_key,
                signature,
            }],
            fingerprint: fingerprint(&account.curve25519_key(), &account.ed25519_key()),
        };

        let encoded = serde_json::to_string(&bundle).unwrap();
        assert!(parse_and_verify_bundle(&encoded, &bundle.fingerprint).is_ok());
    }

    #[test]
    fn fallback_key_can_establish_a_new_session() {
        let alice = Account::new();
        let mut bob = Account::new();
        bob.generate_fallback_key();

        let fallback_key = *bob.fallback_key().values().next().unwrap();

        let mut alice_session = alice
            .create_outbound_session(
                SessionConfig::version_1(),
                bob.curve25519_key(),
                fallback_key,
            )
            .unwrap();

        let OlmMessage::PreKey(pre_key) = alice_session.encrypt("fallback hello").unwrap() else {
            panic!("fallback session must start with a pre-key message");
        };

        let inbound = bob
            .create_inbound_session(SessionConfig::version_1(), alice.curve25519_key(), &pre_key)
            .unwrap();

        assert_eq!(String::from_utf8(inbound.plaintext).unwrap(), "fallback hello");
        assert_eq!(alice_session.session_id(), inbound.session.session_id());

        // The fallback key remains available for another session, unlike an OTK.
        let fallback_again = *bob.fallback_key().values().next().unwrap();
        assert_eq!(fallback_again, fallback_key);
    }

    #[test]
    fn recovery_reestablishes_a_new_olm_session_with_a_prekey() {
        let recovery_key = base64_encode(&[13u8; 32]);
        let mut alice = CryptoDevice::create(&recovery_key).unwrap();
        let mut bob = CryptoDevice::create(&recovery_key).unwrap();

        let alice_bundle = alice.public_bundle_json().unwrap();
        let alice_fingerprint = alice.fingerprint();

        let bob_bundle = bob.public_bundle_json().unwrap();
        let bob_fingerprint = bob.fingerprint();

        // Establish and consume the first session/one-time key.
        alice
            .establish_session(&bob.device_id, &bob_bundle, &bob_fingerprint)
            .unwrap();

        let first = alice.encrypt(&bob.device_id, "before recovery").unwrap();
        assert_eq!(
            bob.decrypt(
                &alice.device_id,
                &first,
                &alice_bundle,
                &alice_fingerprint,
            )
            .unwrap(),
            "before recovery"
        );
        assert_eq!(bob.session_count(&alice.device_id), 1);

        // A recovery response creates a fresh outbound session using a new
        // one-time key. The first message on that session must be a pre-key
        // message, which lets the receiver establish its matching inbound
        // session without knowing anything about the old broken session.
        let refreshed_bob_bundle = bob.public_bundle_json().unwrap();
        alice
            .establish_session(
                &bob.device_id,
                &refreshed_bob_bundle,
                &bob_fingerprint,
            )
            .unwrap();

        let recovered = alice
            .encrypt(&bob.device_id, "recovered payload")
            .unwrap();

        let envelope: super::CipherEnvelope = serde_json::from_str(&recovered).unwrap();
        assert_eq!(envelope.message_type, "prekey");

        assert_eq!(
            bob.decrypt(
                &alice.device_id,
                &recovered,
                &alice_bundle,
                &alice_fingerprint,
            )
            .unwrap(),
            "recovered payload"
        );

        assert_eq!(bob.session_count(&alice.device_id), 2);

        let after_recovery = alice
            .encrypt(&bob.device_id, "after recovery")
            .unwrap();
        let after_envelope: super::CipherEnvelope =
            serde_json::from_str(&after_recovery).unwrap();
        assert_eq!(after_envelope.message_type, "message");

        assert_eq!(
            bob.decrypt(
                &alice.device_id,
                &after_recovery,
                &alice_bundle,
                &alice_fingerprint,
            )
            .unwrap(),
            "after recovery"
        );
    }

    #[test]
    fn delayed_messages_keep_old_sessions() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let mut alice = CryptoDevice::create(&recovery_key).unwrap();
        let mut bob = CryptoDevice::create(&recovery_key).unwrap();

        let alice_bundle = alice.public_bundle_json().unwrap();
        let alice_fingerprint = alice.fingerprint();

        let bob_bundle = bob.public_bundle_json().unwrap();
        let bob_fingerprint = bob.fingerprint();

        alice
            .establish_session(&bob.device_id, &bob_bundle, &bob_fingerprint)
            .unwrap();

        let first = alice.encrypt(&bob.device_id, "first").unwrap();

        assert_eq!(
            bob.decrypt(&alice.device_id, &first, &alice_bundle, &alice_fingerprint)
                .unwrap(),
            "first"
        );

        let reply = bob.encrypt(&alice.device_id, "reply").unwrap();

        assert_eq!(
            alice
                .decrypt(&bob.device_id, &reply, &bob_bundle, &bob_fingerprint)
                .unwrap(),
            "reply"
        );

        let delayed = alice.encrypt(&bob.device_id, "delayed").unwrap();

        assert_eq!(
            bob.decrypt(
                &alice.device_id,
                &delayed,
                &alice_bundle,
                &alice_fingerprint,
            )
            .unwrap(),
            "delayed"
        );
    }

    #[test]
    fn malformed_state_is_rejected() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let result = restore_state_internal(&recovery_key, "{}");

        assert!(result.is_err());
    }

    #[test]
    fn wrong_recovery_key_is_rejected() {
        let recovery_key = base64_encode(&[7u8; 32]);
        let wrong_key = base64_encode(&[8u8; 32]);

        let device = CryptoDevice::create(&recovery_key).unwrap();
        let state = device.encrypted_state_json().unwrap();

        assert!(restore_state_internal(&wrong_key, &state).is_err());
    }

    #[test]
    fn legacy_session_format_is_still_deserializable() {
        let json = r#"{
            "peer": "encrypted-session"
        }"#;

        let sessions: BTreeMap<String, Vec<String>> =
            deserialize_sessions(&mut serde_json::Deserializer::from_str(json)).unwrap();

        assert_eq!(sessions["peer"].len(), 1);
    }

    #[test]
    fn helper_session_decryption_works() {
        let recovery_key = base64_encode(&[7u8; 32]);

        let mut alice = CryptoDevice::create(&recovery_key).unwrap();
        let mut bob = CryptoDevice::create(&recovery_key).unwrap();

        let alice_bundle = alice.public_bundle_json().unwrap();
        let alice_fingerprint = alice.fingerprint();

        let bob_bundle = bob.public_bundle_json().unwrap();
        let bob_fingerprint = bob.fingerprint();

        alice
            .establish_session(&bob.device_id, &bob_bundle, &bob_fingerprint)
            .unwrap();

        let encrypted = alice.encrypt(&bob.device_id, "helper test").unwrap();

        let envelope: super::CipherEnvelope = serde_json::from_str(&encrypted).unwrap();

        assert_eq!(
            bob.decrypt(
                &alice.device_id,
                &encrypted,
                &alice_bundle,
                &alice_fingerprint,
            )
            .unwrap(),
            "helper test"
        );

        let sessions = bob
            .sessions
            .get_mut(&alice.device_id)
            .unwrap_or_else(|| panic!("session should be established"));

        let plaintext = if envelope.message_type == "prekey" {
            decrypt_prekey_with_sessions(sessions, &envelope.ciphertext)
        } else {
            decrypt_with_sessions(sessions, &envelope.ciphertext)
        };

        assert!(plaintext.is_none());
    }
}
