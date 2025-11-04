use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    issue::{LintIssueType, SourceInfoIssue},
    lint_rules::source_info::unknown_architecture::UnknownArchitecture,
};
use alpm_srcinfo::SourceInfo;
use alpm_types::{Architectures, SystemArchitecture};
use rstest::rstest;

use crate::fixtures::default_source_info_v1;

#[test]
fn unknown_architecture_passes() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = Architectures::Some(vec![SystemArchitecture::X86_64]);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnknownArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

#[rstest]
#[case("x86_65", Some("x86_64"))]
#[case("arm64", Some("arm"))]
#[case("skylake", None)]
fn unknown_architecture_fails(
    #[case] value: &str,
    #[case] suggestion: Option<&str>,
) -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures =
        Architectures::Some(vec![SystemArchitecture::from_str(value)?]);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnknownArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::unknown_architecture");

    let LintIssueType::SourceInfo(SourceInfoIssue::Generic { message, .. }) =
        issues[0].issue_type.clone()
    else {
        panic!("Expected a SourceInfoIssue::Generic.");
    };

    if let Some(suggestion) = suggestion {
        assert!(message.contains("Did you mean"));
        assert!(
            message.contains(suggestion),
            "The suggestion '{}' should be in the message.",
            suggestion
        );
    } else {
        assert!(
            !message.contains("Did you mean"),
            "A message should not have suggested a correction."
        );
    }
    Ok(())
}
