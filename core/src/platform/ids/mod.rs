//! Port of `backend/platform/ids`.
//!
//! Generates stable record ids that can be created offline on any machine
//! without collisions.

use rand::RngCore;
use time::OffsetDateTime;

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Returns a 26-character ULID: 48-bit millisecond timestamp followed by
/// 80 random bits, Crockford base32 encoded. Ids sort by creation time.
pub fn new_ulid() -> String {
    ulid_at(OffsetDateTime::now_utc())
}

fn ulid_at(now: OffsetDateTime) -> String {
    let mut raw = [0u8; 16];
    let mut ms = (now.unix_timestamp_nanos() / 1_000_000) as u64;
    for i in (0..6).rev() {
        raw[i] = ms as u8;
        ms >>= 8;
    }
    rand::rng().fill_bytes(&mut raw[6..]);

    // 128 bits → 26 base32 characters (the first character carries 3 bits).
    let mut out = [0u8; 26];
    let mut acc: u64 = 0;
    let mut bits: u32 = 2; // leading padding so 130 bits split evenly into 5-bit groups
    let mut pos = 0;
    for b in raw {
        acc = (acc << 8) | u64::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out[pos] = CROCKFORD[((acc >> bits) & 0x1f) as usize];
            pos += 1;
        }
    }
    String::from_utf8(out.to_vec()).expect("Crockford base32 is ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ulid_shape() {
        let id = new_ulid();
        assert_eq!(id.len(), 26, "len = {}: {id:?}", id.len());
        for r in id.chars() {
            assert!(
                CROCKFORD.contains(&(r as u8)),
                "{id:?} contains {r:?} outside the Crockford alphabet"
            );
        }
    }

    #[test]
    fn new_ulid_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..5000 {
            let id = new_ulid();
            assert!(seen.insert(id.clone()), "duplicate id {id:?}");
        }
    }

    #[test]
    fn ulids_sort_by_creation_time() {
        let base = OffsetDateTime::from_unix_timestamp(1_767_225_600).expect("timestamp"); // 2026-01-01T00:00:00Z
        let earlier = ulid_at(base);
        let later = ulid_at(base + time::Duration::milliseconds(1));
        let much_later = ulid_at(base + time::Duration::hours(48));
        assert!(
            earlier < later && later < much_later,
            "ids must sort by time: {earlier} {later} {much_later}"
        );
    }

    #[test]
    fn ulid_timestamp_prefix_is_stable_within_a_millisecond() {
        let at = OffsetDateTime::from_unix_timestamp(1_780_315_200).expect("timestamp"); // 2026-06-01T12:00:00Z
        let a = ulid_at(at);
        let b = ulid_at(at);
        assert_eq!(&a[..10], &b[..10], "timestamp prefix differs: {a} vs {b}");
    }
}
