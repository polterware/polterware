//! SHA-1, the digest Canon derives entity and relationship ids from, so those
//! ids stay byte for byte stable.

use sha1::{Digest, Sha1};

/// The 20-byte SHA-1 digest of `data`.
pub fn digest(data: &[u8]) -> [u8; 20] {
    Sha1::digest(data).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_matches_the_known_vectors() {
        assert_eq!(
            hex::encode(digest(b"")),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            hex::encode(digest(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            hex::encode(digest(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
    }
}
