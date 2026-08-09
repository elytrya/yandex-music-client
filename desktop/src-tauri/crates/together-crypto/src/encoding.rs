use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use data_encoding::BASE32_NOPAD;

pub fn to_base64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub fn from_base64(text: &str) -> Option<Vec<u8>> {
    STANDARD.decode(text.trim()).ok()
}

pub fn array_from_base64<const N: usize>(text: &str) -> Option<[u8; N]> {
    let raw = from_base64(text)?;
    <[u8; N]>::try_from(raw.as_slice()).ok()
}

pub fn to_base32(bytes: &[u8]) -> String {
    BASE32_NOPAD.encode(bytes).to_lowercase()
}

pub fn from_base32(text: &str) -> Option<Vec<u8>> {
    let cleaned: String = text
        .chars()
        .filter(|symbol| !symbol.is_whitespace() && *symbol != '-')
        .collect();
    BASE32_NOPAD.decode(cleaned.to_uppercase().as_bytes()).ok()
}

pub fn is_base32(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .chars()
            .all(|symbol| symbol.is_ascii_lowercase() || ('2'..='7').contains(&symbol))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip() {
        let raw = [1u8, 2, 3, 250, 251];
        let text = to_base64(&raw);
        assert_eq!(from_base64(&text).as_deref(), Some(&raw[..]));
    }

    #[test]
    fn base64_array_checks_length() {
        let text = to_base64(&[7u8; 32]);
        assert!(array_from_base64::<32>(&text).is_some());
        assert!(array_from_base64::<31>(&text).is_none());
    }

    #[test]
    fn base32_ignores_case_and_dashes() {
        let raw = [9u8, 8, 7, 6, 5, 4];
        let text = to_base32(&raw);
        assert_eq!(text.len(), 10);
        let noisy = format!("{}-{}", &text[..5].to_uppercase(), &text[5..]);
        assert_eq!(from_base32(&noisy).as_deref(), Some(&raw[..]));
    }

    #[test]
    fn base32_alphabet_is_checked() {
        assert!(is_base32("abcdefghij", 10));
        assert!(!is_base32("abcdefghi1", 10));
        assert!(!is_base32("abcdefghi", 10));
    }
}
