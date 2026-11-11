//! Port of `backend/platform/observability`.

use rand::RngCore;

/// Returns a new 128-bit correlation id as 32 lowercase hex characters.
pub fn new_correlation_id() -> String {
    let mut b = [0u8; 16];
    rand::rng().fill_bytes(&mut b);
    hex::encode(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_id() {
        let a = new_correlation_id();
        let b = new_correlation_id();
        assert!(!a.is_empty());
        assert_ne!(a, b);
        assert_eq!(a.len(), 32);
        assert!(
            a.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }
}
