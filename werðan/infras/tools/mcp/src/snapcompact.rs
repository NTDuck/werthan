//! Snapshot compaction and immutable cache stub formatting.

/// Compacted archive manifest with frame hashes and signature.
#[derive(
    ::core::clone::Clone,
    ::core::fmt::Debug,
    ::core::cmp::PartialEq,
    ::core::cmp::Eq,
    ::serde::Serialize,
    ::serde::Deserialize
)]
pub struct SnapCompactArchive {
    /// Ordered frame digest list.
    pub frame_hashes: ::std::vec::Vec<::std::string::String>,
    /// Lead-in banner text.
    pub lead_in_banner: ::std::string::String,
    /// Cryptographic signature for archive verification.
    pub hmac_signature: ::std::string::String,
}

/// Sign manifest hashes and lead-in text with HMAC-SHA256.
pub fn sign_manifest(
    key: &[::core::primitive::u8],
    frame_hashes: &[::std::string::String],
    lead_in: &::core::primitive::str,
) -> ::std::string::String {
    let payload = self::build_manifest_payload(frame_hashes, lead_in);
    let signature_bytes = self::compute_hmac_sha256(key, payload.as_bytes());
    self::hex_encode(&signature_bytes)
}

/// Verify frame byte slice against expected SHA256 hex string.
pub fn verify_frame_bytes(
    expected_sha256_hex: &::core::primitive::str,
    bytes: &[::core::primitive::u8],
) -> ::core::primitive::bool {
    use ::sha2::Digest;
    let mut hasher = ::sha2::Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let computed_hex = self::hex_encode(&digest);
    computed_hex.eq_ignore_ascii_case(expected_sha256_hex)
}

/// Produce deterministic index stub keeping prefix immutable.
pub fn format_text_archive_stub(
    sha256_hex: &::core::primitive::str,
    line_count: ::core::primitive::usize,
) -> ::std::string::String {
    let stub = ::std::format!("[snapcompact:sha256={sha256_hex}:lines={line_count}]");
    stub
}

fn build_manifest_payload(
    frame_hashes: &[::std::string::String],
    lead_in: &::core::primitive::str,
) -> ::std::string::String {
    let mut payload = ::std::string::String::new();
    payload.push_str(lead_in);
    payload.push('\n');
    for hash in frame_hashes {
        payload.push_str(hash);
        payload.push('\n');
    }
    payload
}

fn compute_hmac_sha256(key: &[::core::primitive::u8], data: &[::core::primitive::u8]) -> [::core::primitive::u8; 32] {
    use ::sha2::Digest;
    let mut padded_key = [0u8; 64];
    if key.len() > 64 {
        let mut hasher = ::sha2::Sha256::new();
        hasher.update(key);
        let key_digest = hasher.finalize();
        padded_key[..32].copy_from_slice(&key_digest);
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut inner_pad = [0x36u8; 64];
    let mut outer_pad = [0x5cu8; 64];
    for idx in 0..64 {
        inner_pad[idx] ^= padded_key[idx];
        outer_pad[idx] ^= padded_key[idx];
    }

    let mut inner_hasher = ::sha2::Sha256::new();
    inner_hasher.update(inner_pad);
    inner_hasher.update(data);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = ::sha2::Sha256::new();
    outer_hasher.update(outer_pad);
    outer_hasher.update(inner_hash);
    let outer_hash = outer_hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(&outer_hash);
    result
}

fn hex_encode(bytes: &[::core::primitive::u8]) -> ::std::string::String {
    let mut hex = ::std::string::String::new();
    for &b in bytes {
        use ::core::fmt::Write;
        let _ = ::core::write!(&mut hex, "{:02x}", b);
    }
    hex
}
