//! Tests for hashline crate.

#[::core::prelude::v1::test]
fn test_compute_line_anchor() {
    let lines = ::std::vec!["alpha", "beta", "gamma"];
    let anchor0 = ::harness::hashline::compute_line_anchor(&lines, 0);
    let anchor1 = ::harness::hashline::compute_line_anchor(&lines, 1);
    let anchor2 = ::harness::hashline::compute_line_anchor(&lines, 2);

    ::core::assert_eq!(anchor0.len(), 4);
    ::core::assert_eq!(anchor1.len(), 4);
    ::core::assert_eq!(anchor2.len(), 4);
    ::core::assert_ne!(anchor0, anchor1);
}

#[::core::prelude::v1::test]
fn test_render_hashlined_view() {
    let content = "line 1\nline 2\nline 3\n";
    let view = ::harness::hashline::render_hashlined_view(content, 0, 2);
    ::core::assert!(view.contains("\u{2502} line 1\n"));
    ::core::assert!(view.contains("\u{2502} line 2\n"));
    ::core::assert!(!view.contains("line 3"));
}

#[::core::prelude::v1::test]
fn test_apply_hashline_replace_success() {
    let content = "first\nsecond\nthird\nfourth\n";
    let lines = ::std::vec!["first", "second", "third", "fourth"];
    let a1 = ::harness::hashline::compute_line_anchor(&lines, 1);
    let a2 = ::harness::hashline::compute_line_anchor(&lines, 2);

    let replaced = ::harness::hashline::apply_hashline_replace(content, &a1, &a2, "new_two\nnew_three\n");
    ::core::assert!(replaced.is_ok());
    let res = replaced.unwrap();
    ::core::assert_eq!(res, "first\nnew_two\nnew_three\nfourth\n");
}

#[::core::prelude::v1::test]
fn test_apply_hashline_replace_stale() {
    let content = "first\nsecond\nthird\n";
    let res = ::harness::hashline::apply_hashline_replace(content, "ZZZZ", "YYYY", "new");
    let err_msg = "HASHLINE_STALE_ANCHOR".to_string();
    let expected = ::core::result::Result::Err(err_msg);
    ::core::assert_eq!(res, expected);
}

#[::core::prelude::v1::test]
fn test_apply_hashline_replace_ambiguous() {
    let content = "dup\nmiddle\ndup\n";
    let lines = ::std::vec!["dup", "middle", "dup"];
    let a0 = ::harness::hashline::compute_line_anchor(&lines, 0);
    let a2 = ::harness::hashline::compute_line_anchor(&lines, 2);
    if a0 == a2 {
        let res = ::harness::hashline::apply_hashline_replace(content, &a0, &a0, "replaced");
        let err_msg = "HASHLINE_AMBIGUOUS".to_string();
        let expected = ::core::result::Result::Err(err_msg);
        ::core::assert_eq!(res, expected);
    }
}
