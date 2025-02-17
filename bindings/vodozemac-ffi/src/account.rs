use crate::error::VodozemacError;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uniffi::export;
use vodozemac::olm::{Account as InnerAccount, AccountPickle, Message, PreKeyMessage, SessionConfig, SessionPickle};
use vodozemac::olm::{OlmMessage::Normal, OlmMessage::PreKey};
use vodozemac::olm::Session as InnerSession;
use vodozemac::Curve25519PublicKey;

#[derive(Debug, Deserialize, Serialize, uniffi::Record)]
pub struct AccountIdentityKeys {
    curve25519: String,
    ed25519: String
}

#[derive(Debug, Deserialize, Serialize, uniffi::Record)]
pub struct AccountOneTimeKeys {
    pub one_time_keys: Vec<OneTimeKey>
}

#[derive(Debug, Deserialize, Serialize, uniffi::Record)]
pub struct OneTimeKey {
    pub key_id: String,
    pub value: String
}

#[derive(Debug, Deserialize, Serialize, uniffi::Enum)]
pub enum OlmMessageType {
    PreKey = 0,
    Normal = 1
}

#[derive(Debug, Deserialize, Serialize, uniffi::Record)]
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

    pub fn to_json_session(&self) -> Result<String, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        Ok(serde_json::to_string(&inner.pickle()).unwrap_or_else(|_| "{}".to_string()))
    }

    #[uniffi::constructor]
    fn from(json: String) -> Result<Self, VodozemacError>  {
        let migration_data: SessionPickle = serde_json::from_str(json.as_str())?;
        Ok(Self {
            inner: Mutex::new(InnerSession::from(migration_data)),
        })
    }

    pub fn session_id(&self) -> Result<String, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        Ok(inner.session_id())
    }

    pub fn encrypt(&self, plaintext: String) -> Result<OlmMessage, VodozemacError> {
        let mut inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        let result = inner.encrypt(plaintext.as_bytes());
        match result {
            Normal(msg) => {
                Ok(OlmMessage {
                    ciphertext: msg.to_base64(),
                    message_type: OlmMessageType::Normal,
                })
            }
            PreKey(msg) => {
                Ok(OlmMessage {
                    ciphertext: msg.to_base64(),
                    message_type: OlmMessageType::PreKey,
                })
            }
        }
    }

    pub fn decrypt(&self, message: OlmMessage) -> Result<String, VodozemacError> {
        let mut inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        match message.message_type {
            OlmMessageType::PreKey => {
                let pk_msg = PreKeyMessage::from_base64(&message.ciphertext)?;
                let m = PreKey(pk_msg);
                let result = inner.decrypt(&m)?;
                let str = String::from_utf8(result)?;
                Ok(str)
            }
            OlmMessageType::Normal => {
                let msg = Message::from_base64(&message.ciphertext)?;
                let m = Normal(msg);
                let result = inner.decrypt(&m)?;
                let str = String::from_utf8(result)?;
                Ok(str)
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

    #[uniffi::constructor]
    fn from(json: String) -> Result<Self, VodozemacError>  {
        let migration_data: AccountPickle = serde_json::from_str(json.as_str())?;
        Ok(Self {
            inner: Mutex::new(InnerAccount::from(migration_data)),
        })
    }

    pub fn to_json_account(&self) -> Result<String, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        Ok(serde_json::to_string(&inner.pickle()).unwrap_or_else(|_| "{}".to_string()))
    }

    pub fn sign(&self, message: String) -> Result<String, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        let signature = inner.sign(message);
        Ok(signature.to_base64())
    }

    pub fn identity_keys(&self) -> Result<AccountIdentityKeys, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        let identity_keys = inner.identity_keys();
        let curve_key = identity_keys.curve25519.to_base64();
        let ed25519_key = identity_keys.ed25519.to_base64();

        Ok(AccountIdentityKeys {
            curve25519: curve_key,
            ed25519: ed25519_key
        })
    }

    pub fn mark_keys_as_published(&self) -> Result<(), VodozemacError> {
        let mut inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        Ok(inner.mark_keys_as_published())
    }

    pub fn generate_one_time_keys(&self, count: u32) -> Result<(), VodozemacError> {
        let mut inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        inner.generate_one_time_keys(count as usize);
        Ok(())
    }

    pub fn one_time_keys(&self) -> Result<AccountOneTimeKeys, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        let keys = inner.one_time_keys()
            .iter()
            .map(|(k, v)| (k.to_base64(), v.to_base64()))
            .map(|(k, v)| OneTimeKey { key_id: k, value: v }).collect();

        Ok(AccountOneTimeKeys { one_time_keys: keys })
    }

    pub fn outbound_session(&self, identity_key: String, one_time_key: String) -> Result<Session, VodozemacError> {
        let inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        let base64_identity_key = Curve25519PublicKey::from_base64(&identity_key)?;
        let base64_one_time_key = Curve25519PublicKey::from_base64(&one_time_key)?;

        Ok(Session {
            inner: Mutex::new(inner.create_outbound_session(
                SessionConfig::default(),
                base64_identity_key,
                base64_one_time_key,
            ))
        })
    }

    pub fn create_inbound_session(&self, their_identity_key: String, pre_key_message: String) -> Result<CreateInboundSessionResult, VodozemacError> {
        let mut inner = self.inner.lock().map_err(|_| VodozemacError::LockError)?;
        let key = Curve25519PublicKey::from_base64(&their_identity_key)?;
        let pk = PreKeyMessage::from_base64(&pre_key_message)?;
        let result = inner.create_inbound_session(key, &pk)?;
        let message = String::from_utf8(result.plaintext)?;

        Ok(CreateInboundSessionResult {
            session: Arc::from(Session { inner: Mutex::new(result.session) }),
            message
        })
    }
}