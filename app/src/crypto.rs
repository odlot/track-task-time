use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::model::Store;

const ENVELOPE_VERSION: u32 = 1;
const KDF_NAME: &str = "argon2id";
const CIPHER_NAME: &str = "xchacha20poly1305";
const SALT_LEN: usize = 16;

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedStore {
    version: u32,
    kdf: KdfConfig,
    cipher: String,
    salt: String,
    nonce: String,
    ciphertext: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KdfConfig {
    name: String,
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
}

pub fn read_passphrase(confirm: bool) -> Result<String, String> {
    let passphrase = rpassword::prompt_password("Passphrase: ").map_err(|err| err.to_string())?;
    if passphrase.trim().is_empty() {
        return Err("Passphrase cannot be empty.".into());
    }

    if confirm {
        let confirm_passphrase =
            rpassword::prompt_password("Confirm passphrase: ").map_err(|err| err.to_string())?;
        if passphrase != confirm_passphrase {
            return Err("Passphrases do not match.".into());
        }
    }

    Ok(passphrase)
}

pub fn encrypt_store(store: &Store, passphrase: &str) -> Result<String, String> {
    if passphrase.trim().is_empty() {
        return Err("Passphrase cannot be empty.".into());
    }

    let payload = serde_json::to_vec(store).map_err(|err| err.to_string())?;
    let kdf = default_kdf();
    let salt = random_bytes(SALT_LEN);
    let key = derive_key(passphrase, &salt, &kdf)?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce_bytes = random_bytes(24);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, payload.as_ref())
        .map_err(|_| "Encryption failed.".to_string())?;

    let envelope = EncryptedStore {
        version: ENVELOPE_VERSION,
        kdf,
        cipher: CIPHER_NAME.to_string(),
        salt: general_purpose::STANDARD.encode(salt),
        nonce: general_purpose::STANDARD.encode(nonce_bytes),
        ciphertext: general_purpose::STANDARD.encode(ciphertext),
    };

    serde_json::to_string_pretty(&envelope).map_err(|err| err.to_string())
}

pub fn decrypt_store(payload: &str, passphrase: &str) -> Result<Store, String> {
    if passphrase.trim().is_empty() {
        return Err("Passphrase cannot be empty.".into());
    }

    let envelope: EncryptedStore = serde_json::from_str(payload).map_err(|err| err.to_string())?;
    if envelope.version != ENVELOPE_VERSION {
        return Err(format!("Unsupported data version {}.", envelope.version));
    }
    if envelope.cipher != CIPHER_NAME {
        return Err(format!("Unsupported cipher {}.", envelope.cipher));
    }
    if envelope.kdf.name != KDF_NAME {
        return Err(format!("Unsupported KDF {}.", envelope.kdf.name));
    }

    let salt = general_purpose::STANDARD
        .decode(envelope.salt)
        .map_err(|_| "Invalid salt encoding.".to_string())?;
    let nonce_bytes = general_purpose::STANDARD
        .decode(envelope.nonce)
        .map_err(|_| "Invalid nonce encoding.".to_string())?;
    let ciphertext = general_purpose::STANDARD
        .decode(envelope.ciphertext)
        .map_err(|_| "Invalid ciphertext encoding.".to_string())?;
    if salt.is_empty() {
        return Err("Invalid salt length.".into());
    }
    if nonce_bytes.len() != 24 {
        return Err("Invalid nonce length.".into());
    }

    let key = derive_key(passphrase, &salt, &envelope.kdf)?;
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let nonce = XNonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Invalid passphrase or corrupted data file.".to_string())?;

    serde_json::from_slice(&plaintext).map_err(|err| err.to_string())
}

fn default_kdf() -> KdfConfig {
    KdfConfig {
        name: KDF_NAME.to_string(),
        m_cost: 19_456,
        t_cost: 2,
        p_cost: 1,
    }
}

fn derive_key(passphrase: &str, salt: &[u8], kdf: &KdfConfig) -> Result<[u8; 32], String> {
    let params =
        Params::new(kdf.m_cost, kdf.t_cost, kdf.p_cost, None).map_err(|err| err.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|err| err.to_string())?;
    Ok(key)
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Segment, Task};

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let store = Store {
            version: 1,
            tasks: vec![Task {
                id: "id".into(),
                name: "Task".into(),
                created_at: chrono::Utc::now(),
                closed_at: None,
                segments: vec![Segment {
                    start_at: chrono::Utc::now(),
                    end_at: None,
                }],
            }],
        };

        let payload = encrypt_store(&store, "secret-passphrase").unwrap();
        let decoded = decrypt_store(&payload, "secret-passphrase").unwrap();
        assert_eq!(decoded.tasks.len(), 1);
        assert_eq!(decoded.tasks[0].name, "Task");
    }
}
