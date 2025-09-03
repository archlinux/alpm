//! Integration tests for [`LintIssueDisplay`] formatting.
//!
//! These tests verify that the display output is correctly formatted via insta snapshots

use std::collections::BTreeMap;

use alpm_lint::{Level, issue::display::LintIssueDisplay};
use rstest::rstest;

/// Helper function to create a default LintIssueDisplay for testing
fn default_display() -> LintIssueDisplay {
    LintIssueDisplay {
        level: Level::Error,
        scoped_name: "lint_rule_name".to_string(),
        summary: None,
        arrow_line: None,
        message: "message".to_string(),
        help_text: "help_text".to_string(),
        custom_links: BTreeMap::new(),
    }
}

/// Helper function to disable colored output for consistent snapshots
fn force_color_off() {
    colored::control::set_override(false);
}

/// Test each severity level with basic formatting
#[rstest]
#[case(Level::Error)]
#[case(Level::Deny)]
#[case(Level::Warn)]
#[case(Level::Suggest)]
fn test_severity_levels(#[case] level: Level) {
    force_color_off();

    let display = LintIssueDisplay {
        level,
        ..default_display()
    };

    let test_name = level.to_string().to_lowercase();

    insta::with_settings!({
        description => format!("{test_name} severity level display."),
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(format!("{test_name}_basic"), format!("{display}"));
    });
}

/// Test display with summary line
#[test]
fn test_display_with_summary() {
    force_color_off();

    let display = LintIssueDisplay {
        summary: Some("summary".to_string()),
        ..default_display()
    };

    insta::with_settings!({
        description => "Display with summary line.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("with_summary", format!("{display}"));
    });
}

/// Test display with arrow line context
#[test]
fn test_display_with_arrow_line() {
    force_color_off();

    let display = LintIssueDisplay {
        arrow_line: Some("arrow_line".to_string()),
        ..default_display()
    };

    insta::with_settings!({
        description => "Display with arrow line context.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("with_arrow_line", format!("{display}"));
    });
}

/// Test display with multi-line message
#[test]
fn test_display_with_multiline_message() {
    force_color_off();

    let display = LintIssueDisplay {
        message: "multi_line\nmessage".to_string(),
        ..default_display()
    };

    insta::with_settings!({
        description => "Display with multi-line message content.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("multiline_message", format!("{display}"));
    });
}

/// Test display with multi-line help text
#[test]
fn test_display_with_multiline_help() {
    force_color_off();

    let display = LintIssueDisplay {
        help_text: "multi_line\nhelp_text".to_string(),
        ..default_display()
    };

    insta::with_settings!({
        description => "Display with multi-line help text.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("multiline_help", format!("{display}"));
    });
}

/// Test display with custom links
#[test]
fn test_display_with_custom_links() {
    force_color_off();

    let mut custom_links = BTreeMap::new();
    custom_links.insert("link_name_1".to_string(), "url_1".to_string());
    custom_links.insert("link_name_2".to_string(), "url_2".to_string());

    let display = LintIssueDisplay {
        custom_links,
        ..default_display()
    };

    insta::with_settings!({
        description => "Display with custom links.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("with_custom_links", format!("{display}"));
    });
}

/// Test display with all everything.
#[test]
fn test_display_everything() {
    force_color_off();

    let mut custom_links = BTreeMap::new();
    custom_links.insert("link_name_1".to_string(), "url_1".to_string());
    custom_links.insert("link_name_2".to_string(), "url_2".to_string());

    let display = LintIssueDisplay {
        summary: Some("summary".to_string()),
        arrow_line: Some("arrow_line".to_string()),
        message: "multi_line\nmessage".to_string(),
        help_text: "multi_line\nhelp_text".to_string(),
        custom_links,
        ..default_display()
    };

    insta::with_settings!({
        description => "Display with everything enabled.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("everything", format!("{display}"));
    });
}

/// Test minimal display
#[test]
fn test_display_minimal() {
    force_color_off();

    let display = default_display();

    insta::with_settings!({
        description => "Minimal display with only required fields.",
        snapshot_path => "display_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!("minimal", format!("{display}"));
    });
}
