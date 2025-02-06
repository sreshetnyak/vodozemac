use uniffi::export;
use vodozemac::olm::{Account as InnerAccount, Message, PreKeyMessage, SessionConfig};
use vodozemac::olm::{Session as InnerSession};
use vodozemac::olm::{OlmMessage::Normal, OlmMessage::PreKey};
use std::sync::{Arc, Mutex};
use vodozemac::{Curve25519PublicKey};

#[derive(uniffi::Record)]
pub struct AccountIdentityKeys {
    curve25519: String,
    ed25519: String
}

#[derive(uniffi::Record)]
pub struct AccountOneTimeKeys {
    pub one_time_keys: Vec<OneTimeKey>
}

#[derive(uniffi::Record)]
pub struct OneTimeKey {
    pub key_id: String,
    pub value: String
}

#[derive(uniffi::Enum)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OlmMessageType {
    PreKey = 0,
    Normal = 1
}

#[derive(uniffi::Record)]
pub struct OlmMessage {
    pub ciphertext: String,
    pub message_type: OlmMessageType
}

#[derive(uniffi::Object)]
pub struct CreateInboundSessionResult {
    session: Arc<Session>,
    message: String
}

#[export]
impl CreateInboundSessionResult {

    pub fn get_session(&self) -> Arc<Session> {
        Arc::clone(&self.session)
    }

    pub fn get_message(&self) -> String {
        self.message.clone()
    }
}

#[derive(uniffi::Object)]
pub struct Session {
    pub(crate) inner: Mutex<InnerSession>
}

#[export]
impl Session {

    pub fn session_id(&self) -> String {
        let inner = self.inner.lock().unwrap();
        inner.session_id()
    }

    pub fn encrypt(&self, plaintext: String) -> OlmMessage {
        let mut inner = self.inner.lock().unwrap();
        let result = inner.encrypt(plaintext.as_bytes());
        match result {
            Normal(msg) => {
                OlmMessage {
                    ciphertext: msg.to_base64(),
                    message_type: OlmMessageType::Normal,
                }
            }
            PreKey(msg) => {
                OlmMessage {
                    ciphertext: msg.to_base64(),
                    message_type: OlmMessageType::PreKey,
                }
            }
        }
    }

    pub fn decrypt(&self, message: OlmMessage) -> String {
        let mut inner = self.inner.lock().unwrap();
        match message.message_type {
            OlmMessageType::PreKey => {
                let m = PreKey(PreKeyMessage::from_base64(&message.ciphertext).unwrap());
                let result = inner.decrypt(&m).unwrap();
                String::from_utf8(result).unwrap()
            }
            OlmMessageType::Normal => {
                let m = Normal(Message::from_base64(&message.ciphertext).unwrap());
                let result = inner.decrypt(&m).unwrap();
                String::from_utf8(result).unwrap()
            }
        }
    }
}

#[derive(uniffi::Object)]
pub struct Account {
    pub(crate) inner: Mutex<InnerAccount>
}

#[export]
impl Account {

    #[uniffi::constructor]
    fn new() -> Self {
        Self {
            inner: Mutex::new(InnerAccount::default()),
        }
    }

    pub fn identity_keys(&self) -> AccountIdentityKeys {
        let inner = self.inner.lock().unwrap();
        let identity_keys = inner.identity_keys();
        let curve_key = identity_keys.curve25519.to_base64();
        let ed25519_key = identity_keys.ed25519.to_base64();
        AccountIdentityKeys { curve25519: curve_key, ed25519: ed25519_key }
    }

    pub fn mark_keys_as_published(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.mark_keys_as_published()
    }

    pub fn generate_one_time_keys(&self, count: u32) {
        let mut inner = self.inner.lock().unwrap();
        inner.generate_one_time_keys(count as usize);
    }

    pub fn one_time_keys(&self) -> AccountOneTimeKeys {
        let inner = self.inner.lock().unwrap();
        let keys = inner.one_time_keys()
            .iter()
            .map(|(k, v)| (k.to_base64(), v.to_base64()))
            .map(|(k, v)| OneTimeKey { key_id: k, value: v }).collect();

        AccountOneTimeKeys { one_time_keys: keys }
    }

    pub fn outbound_session(&self, identity_key: String, one_time_key: String) -> Session {
        let inner = self.inner.lock().unwrap();
        Session {
            inner: Mutex::new(inner.create_outbound_session(
                SessionConfig::default(),
                Curve25519PublicKey::from_base64(&identity_key).unwrap(),
                Curve25519PublicKey::from_base64(&one_time_key).unwrap(),
            ))
        }
    }

    pub fn create_inbound_session(&self, their_identity_key: String, pre_key_message: String) -> CreateInboundSessionResult {
        let mut inner = self.inner.lock().unwrap();
        let key = Curve25519PublicKey::from_base64(&their_identity_key).unwrap();
        let pk = PreKeyMessage::from_base64(&pre_key_message).unwrap();
        let result = inner.create_inbound_session(key, &pk).unwrap();
        CreateInboundSessionResult {
            session: Arc::from(Session { inner: Mutex::new(result.session) }),
            message: String::from_utf8(result.plaintext).unwrap()
        }
    }
}