use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use vodozemac::olm::{
    Account, AccountPickle, Message, OlmMessage, PreKeyMessage, Session, SessionConfig,
    SessionPickle,
};
use vodozemac::{base64_decode, Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublicBundle {
    curve_key: String,
    ed25519_key: String,
    identity_signature: String,
    one_time_key: String,
    one_time_signature: String,
    fingerprint: String,
}

#[derive(Serialize, Deserialize)]
struct CipherEnvelope {
    version: u8,
    message_type: String,
    ciphertext: String,
}

#[derive(Serialize, Deserialize)]
struct PersistedState {
    account_pickle: String,
    #[serde(deserialize_with = "deserialize_sessions")]
    sessions: BTreeMap<String, Vec<String>>,
    one_time_key: String,
    one_time_signature: String,
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

#[wasm_bindgen]
pub struct CryptoDevice {
    account: Account,
    sessions: BTreeMap<String, Vec<Session>>,
    recovery_key: [u8; 32],
    one_time_key: String,
    one_time_signature: String,
}

#[wasm_bindgen]
impl CryptoDevice {
    #[wasm_bindgen(constructor)]
    pub fn create(recovery_key_base64: &str) -> Result<CryptoDevice, JsValue> {
        let recovery_key = decode_recovery_key(recovery_key_base64)?;
        let mut account = Account::new();
        account.generate_one_time_keys(1);
        let (_, one_time_key) = account
            .one_time_keys()
            .into_iter()
            .next()
            .ok_or_else(|| JsValue::from_str("could not generate one-time key"))?;
        let one_time_signature = account.sign(one_time_key.as_bytes()).to_base64();
        account.mark_keys_as_published();
        Ok(Self {
            account,
            sessions: BTreeMap::new(),
            recovery_key,
            one_time_key: one_time_key.to_base64(),
            one_time_signature,
        })
    }

    #[wasm_bindgen(js_name = restore)]
    pub fn restore(recovery_key_base64: &str, state_json: &str) -> Result<CryptoDevice, JsValue> {
        let recovery_key = decode_recovery_key(recovery_key_base64)?;
        let state: PersistedState = serde_json::from_str(state_json)
            .map_err(|_| JsValue::from_str("invalid encrypted crypto state"))?;
        let account_pickle = AccountPickle::from_encrypted(&state.account_pickle, &recovery_key)
            .map_err(|_| JsValue::from_str("wrong recovery key or damaged crypto state"))?;
        let account = Account::from(account_pickle);
        let sessions = state
            .sessions
            .into_iter()
            .map(|(peer_id, ciphertexts)| {
                let sessions = ciphertexts
                    .into_iter()
                    .map(|ciphertext| {
                        let pickle = SessionPickle::from_encrypted(&ciphertext, &recovery_key)
                            .map_err(|_| {
                                JsValue::from_str("could not restore encrypted session")
                            })?;
                        Ok(Session::from(pickle))
                    })
                    .collect::<Result<Vec<_>, JsValue>>()?;
                Ok((peer_id, sessions))
            })
            .collect::<Result<BTreeMap<_, _>, JsValue>>()?;
        Ok(Self {
            account,
            sessions,
            recovery_key,
            one_time_key: state.one_time_key,
            one_time_signature: state.one_time_signature,
        })
    }

    pub fn fingerprint(&self) -> String {
        fingerprint(&self.account.curve25519_key(), &self.account.ed25519_key())
    }

    pub fn public_bundle_json(&self) -> Result<String, JsValue> {
        let curve_key = self.account.curve25519_key();
        let ed25519_key = self.account.ed25519_key();
        let identity_signature = self.account.sign(curve_key.as_bytes()).to_base64();
        serde_json::to_string(&PublicBundle {
            curve_key: curve_key.to_base64(),
            ed25519_key: ed25519_key.to_base64(),
            identity_signature,
            one_time_key: self.one_time_key.clone(),
            one_time_signature: self.one_time_signature.clone(),
            fingerprint: fingerprint(&curve_key, &ed25519_key),
        })
        .map_err(|_| JsValue::from_str("could not encode public key bundle"))
    }

    pub fn has_session(&self, peer_id: &str) -> bool {
        self.sessions
            .get(peer_id)
            .is_some_and(|sessions| !sessions.is_empty())
    }

    pub fn session_count(&self, peer_id: &str) -> usize {
        self.sessions
            .get(peer_id)
            .map_or(0, Vec::len)
    }

    pub fn establish_session(
        &mut self,
        peer_id: &str,
        bundle_json: &str,
        expected_fingerprint: &str,
    ) -> Result<(), JsValue> {
        let bundle = parse_and_verify_bundle(bundle_json, expected_fingerprint)?;
        let identity_key = Curve25519PublicKey::from_base64(&bundle.curve_key)
            .map_err(|_| JsValue::from_str("invalid peer identity key"))?;
        let one_time_key = Curve25519PublicKey::from_base64(&bundle.one_time_key)
            .map_err(|_| JsValue::from_str("invalid peer one-time key"))?;
        let session = self
            .account
            .create_outbound_session(SessionConfig::version_1(), identity_key, one_time_key)
            .map_err(|_| JsValue::from_str("could not establish encrypted session"))?;
        self.sessions
            .entry(peer_id.to_string())
            .or_default()
            .push(session);
        Ok(())
    }

    pub fn encrypt(&mut self, peer_id: &str, plaintext: &str) -> Result<String, JsValue> {
        let session = self
            .sessions
            .get_mut(peer_id)
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
            version: 1,
            message_type: message_type.to_string(),
            ciphertext,
        })
        .map_err(|_| JsValue::from_str("could not encode encrypted message"))
    }

    pub fn decrypt(
        &mut self,
        peer_id: &str,
        envelope_json: &str,
        sender_bundle_json: &str,
        expected_fingerprint: &str,
    ) -> Result<String, JsValue> {
        let envelope: CipherEnvelope = serde_json::from_str(envelope_json)
            .map_err(|_| JsValue::from_str("invalid encrypted message envelope"))?;
        if envelope.version != 1 {
            return Err(JsValue::from_str("unsupported encrypted message version"));
        }
        if envelope.message_type == "prekey" {
            let bundle = parse_and_verify_bundle(sender_bundle_json, expected_fingerprint)?;
            let sender_identity = Curve25519PublicKey::from_base64(&bundle.curve_key)
                .map_err(|_| JsValue::from_str("invalid sender identity key"))?;
            let pre_key = PreKeyMessage::from_base64(&envelope.ciphertext)
                .map_err(|_| JsValue::from_str("invalid encrypted pre-key message"))?;
            if pre_key.identity_key() != sender_identity {
                return Err(JsValue::from_str(
                    "sender identity does not match verified key",
                ));
            }
            if let Some(sessions) = self.sessions.get_mut(peer_id) {
                if let Some(plaintext) =
                    decrypt_prekey_with_sessions(sessions, &envelope.ciphertext)
                {
                    return String::from_utf8(plaintext)
                        .map_err(|_| JsValue::from_str("decrypted message is not valid UTF-8"));
                }
            }
            let result = self
                .account
                .create_inbound_session(SessionConfig::version_1(), sender_identity, &pre_key)
                .map_err(|_| JsValue::from_str("could not establish incoming encrypted session"))?;
            let plaintext = String::from_utf8(result.plaintext)
                .map_err(|_| JsValue::from_str("decrypted message is not valid UTF-8"))?;
            self.sessions
                .entry(peer_id.to_string())
                .or_default()
                .push(result.session);
            self.account.generate_one_time_keys(1);
            let (_, next_key) = self
                .account
                .one_time_keys()
                .into_iter()
                .next()
                .ok_or_else(|| JsValue::from_str("could not replenish one-time key"))?;
            self.one_time_signature = self.account.sign(next_key.as_bytes()).to_base64();
            self.one_time_key = next_key.to_base64();
            self.account.mark_keys_as_published();
            return Ok(plaintext);
        }
        if envelope.message_type != "message" {
            return Err(JsValue::from_str("unsupported encrypted message type"));
        }
        let sessions = self
            .sessions
            .get_mut(peer_id)
            .ok_or_else(|| JsValue::from_str("encrypted session is not established"))?;
        let plaintext = decrypt_with_sessions(sessions, &envelope.ciphertext).ok_or_else(|| {
            JsValue::from_str("could not decrypt message with any existing session")
        })?;
        String::from_utf8(plaintext)
            .map_err(|_| JsValue::from_str("decrypted message is not valid UTF-8"))
    }

    pub fn encrypted_state_json(&self) -> Result<String, JsValue> {
        let account_pickle = self.account.pickle().encrypt(&self.recovery_key);
        let sessions = self
            .sessions
            .iter()
            .map(|(peer_id, sessions)| {
                (
                    peer_id.clone(),
                    sessions
                        .iter()
                        .map(|session| session.pickle().encrypt(&self.recovery_key))
                        .collect(),
                )
            })
            .collect();
        serde_json::to_string(&PersistedState {
            account_pickle,
            sessions,
            one_time_key: self.one_time_key.clone(),
            one_time_signature: self.one_time_signature.clone(),
        })
        .map_err(|_| JsValue::from_str("could not encrypt crypto state"))
    }
}

fn decrypt_with_sessions(sessions: &mut [Session], ciphertext: &str) -> Option<Vec<u8>> {
    let message = Message::from_base64(ciphertext).ok()?;

    for session in sessions.iter_mut().rev() {
        if let Ok(plaintext) = session.decrypt(&OlmMessage::Normal(message.clone())) {
            return Some(plaintext);
        }
    }

    None
}

fn decrypt_prekey_with_sessions(sessions: &mut [Session], ciphertext: &str) -> Option<Vec<u8>> {
    for session in sessions.iter_mut().rev() {
        let message = PreKeyMessage::from_base64(ciphertext).ok()?;
        if let Ok(plaintext) = session.decrypt(&OlmMessage::PreKey(message)) {
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
    let one_time_key = Curve25519PublicKey::from_base64(&bundle.one_time_key)
        .map_err(|_| JsValue::from_str("invalid one-time key"))?;
    let one_time_signature = Ed25519Signature::from_base64(&bundle.one_time_signature)
        .map_err(|_| JsValue::from_str("invalid one-time key signature"))?;
    ed_key
        .verify(one_time_key.as_bytes(), &one_time_signature)
        .map_err(|_| JsValue::from_str("one-time key signature verification failed"))?;
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
    use std::collections::BTreeMap;

    use super::{
        decrypt_prekey_with_sessions, decrypt_with_sessions, deserialize_sessions, CryptoDevice,
    };
    use serde::Deserialize;
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
    fn fresh_prekey_keeps_older_sessions_for_delayed_messages() {
        let recovery_key = base64_encode(&[7u8; 32]);
        let mut alice = CryptoDevice::create(&recovery_key).unwrap();
        let mut bob = CryptoDevice::create(&recovery_key).unwrap();
        let alice_bundle = alice.public_bundle_json().unwrap();
        let alice_fingerprint = alice.fingerprint();
        let bob_bundle = bob.public_bundle_json().unwrap();
        let bob_fingerprint = bob.fingerprint();

        alice
            .establish_session("bob", &bob_bundle, &bob_fingerprint)
            .unwrap();
        let first_prekey = alice.encrypt("bob", "first session").unwrap();
        assert_eq!(
            bob.decrypt("alice", &first_prekey, &alice_bundle, &alice_fingerprint)
                .unwrap(),
            "first session"
        );
        let repeated_prekey = alice
            .encrypt("bob", "another message before reply")
            .unwrap();
        let repeated_envelope: super::CipherEnvelope =
            serde_json::from_str(&repeated_prekey).unwrap();
        assert_eq!(repeated_envelope.message_type, "prekey");
        assert_eq!(
            String::from_utf8(
                decrypt_prekey_with_sessions(
                    bob.sessions.get_mut("alice").unwrap(),
                    &repeated_envelope.ciphertext,
                )
                .unwrap(),
            )
            .unwrap(),
            "another message before reply"
        );

        let reply = bob.encrypt("alice", "session reply").unwrap();
        assert_eq!(
            alice
                .decrypt("bob", &reply, &bob_bundle, &bob_fingerprint)
                .unwrap(),
            "session reply"
        );
        let delayed_message = alice.encrypt("bob", "delayed on old session").unwrap();
        let delayed_envelope: super::CipherEnvelope =
            serde_json::from_str(&delayed_message).unwrap();
        assert_eq!(delayed_envelope.message_type, "message");

        let refreshed_bob_bundle = bob.public_bundle_json().unwrap();
        alice
            .establish_session("bob", &refreshed_bob_bundle, &bob_fingerprint)
            .unwrap();
        let second_prekey = alice.encrypt("bob", "second session").unwrap();
        assert_eq!(
            bob.decrypt("alice", &second_prekey, &alice_bundle, &alice_fingerprint)
                .unwrap(),
            "second session"
        );

        let saved_state = bob.encrypted_state_json().unwrap();
        let mut restored_bob = CryptoDevice::restore(&recovery_key, &saved_state).unwrap();
        assert_eq!(
            String::from_utf8(
                decrypt_with_sessions(
                    restored_bob.sessions.get_mut("alice").unwrap(),
                    &delayed_envelope.ciphertext,
                )
                .unwrap(),
            )
            .unwrap(),
            "delayed on old session"
        );

        #[derive(Deserialize)]
        struct LegacyState {
            #[serde(deserialize_with = "deserialize_sessions")]
            sessions: BTreeMap<String, Vec<String>>,
        }
        let legacy: LegacyState =
            serde_json::from_str(r#"{"sessions":{"alice":"old-pickle"}}"#).unwrap();
        assert_eq!(legacy.sessions["alice"], ["old-pickle"]);
    }
}
