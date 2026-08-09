use ed25519_dalek::SigningKey;
use keyring::Entry;
use together_crypto::encoding::to_base64;
use together_crypto::identity;

pub const SERVICE: &str = "mashiro-together";

pub const ACCOUNT: &str = "seed-phrase";

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE, ACCOUNT).map_err(|error| format!("хранилище ключей недоступно: {error}"))
}

pub fn save(phrase: &str) -> Result<String, String> {
    let normalized = identity::normalize_phrase(phrase);
    identity::keypair_from_phrase(&normalized).map_err(|error| error.to_string())?;

    entry()?
        .set_password(&normalized)
        .map_err(|error| format!("не удалось сохранить сид-фразу: {error}"))?;

    Ok(normalized)
}

pub fn load() -> Result<Option<String>, String> {
    match entry()?.get_password() {
        Ok(phrase) => Ok(Some(phrase)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("не удалось прочитать хранилище ключей: {error}")),
    }
}

pub fn forget() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("не удалось удалить сид-фразу: {error}")),
    }
}

pub fn create(words: usize) -> Result<String, String> {
    let phrase = identity::generate_phrase(words).map_err(|error| error.to_string())?;
    save(&phrase)
}

pub fn signing_key(phrase: &str) -> Result<SigningKey, String> {
    identity::keypair_from_phrase(phrase).map_err(|error| error.to_string())
}

pub fn public(phrase: &str) -> Result<String, String> {
    Ok(to_base64(&signing_key(phrase)?.verifying_key().to_bytes()))
}
