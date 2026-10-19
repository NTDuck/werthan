//! Tests for snapcompact crate.

#[::core::prelude::v1::test]
fn test_sign_manifest() {
    let key = b"secret-test-key";
    let hashes = ::std::vec!["abcd".to_string(), "1234".to_string()];
    let lead_in = "banner";
    let sig1 = ::harness::snapcompact::sign_manifest(key, &hashes, lead_in);
    let sig2 = ::harness::snapcompact::sign_manifest(key, &hashes, lead_in);
    ::core::assert_eq!(sig1, sig2);
    ::core::assert_eq!(sig1.len(), 64);
}

#[::core::prelude::v1::test]
fn test_verify_frame_bytes() {
    let bytes = b"hello world";
    let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
    let matches = ::harness::snapcompact::verify_frame_bytes(expected, bytes);
    ::core::assert!(matches);

    let wrong = ::harness::snapcompact::verify_frame_bytes("00000000", bytes);
    ::core::assert!(!wrong);
}

#[::core::prelude::v1::test]
fn test_format_text_archive_stub() {
    let stub = ::harness::snapcompact::format_text_archive_stub("feedface", 42);
    ::core::assert_eq!(stub, "[snapcompact:sha256=feedface:lines=42]");
}
