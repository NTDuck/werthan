//! Tests for tui layout and functions.

#[test]
fn test_compute_layout_6040() {
    let area = ::ratatui::layout::Rect::new(0, 0, 100, 50);
    let (left, right_panes) = ::interactive::compute_layout(area);

    ::std::assert_eq!(left.width, 60);
    ::std::assert_eq!(left.height, 50);

    ::std::assert_eq!(right_panes.len(), 5);
    for pane in right_panes {
        ::std::assert_eq!(pane.x, 60);
        ::std::assert_eq!(pane.width, 40);
        ::std::assert_eq!(pane.height, 10);
    }
}

#[test]
fn test_is_input_locked_to_left() {
    let locked_10 = ::interactive::is_input_locked_to_left(10, 60);
    ::std::assert!(locked_10);
    let locked_59 = ::interactive::is_input_locked_to_left(59, 60);
    ::std::assert!(locked_59);
    let locked_60 = ::interactive::is_input_locked_to_left(60, 60);
    ::std::assert!(!locked_60);
    let locked_75 = ::interactive::is_input_locked_to_left(75, 60);
    ::std::assert!(!locked_75);
}

#[test]
fn test_highlight_magic_keywords() {
    let prompt = "please ultrathink and orchestrate the workflowz run";
    let tokens = ::interactive::highlight_magic_keywords(prompt);

    let magic_tokens = tokens
        .into_iter()
        .filter(|(_, is_magic)| *is_magic)
        .map(|(w, _)| w)
        .collect::<::std::vec::Vec<::std::string::String>>();

    ::std::assert_eq!(
        magic_tokens,
        ::std::vec![
            ::std::string::String::from("ultrathink"),
            ::std::string::String::from("orchestrate"),
            ::std::string::String::from("workflowz")
        ]
    );
}

#[test]
fn test_csi_sync_constants() {
    ::std::assert_eq!(::interactive::CSI_SYNC_START, "\x1b[?2026h");
    ::std::assert_eq!(::interactive::CSI_SYNC_END, "\x1b[?2026l");
}
