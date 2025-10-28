use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::no_architecture::NoArchitecture,
};
use alpm_srcinfo::SourceInfo;
use alpm_types::{Architectures, SystemArchitecture};

use crate::fixtures::default_source_info_v1;

#[test]
fn no_architecture_passes() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = Architectures::Some(vec![SystemArchitecture::X86_64]);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = NoArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

#[test]
fn no_architecture_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = Architectures::Some(vec![]);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = NoArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::no_architecture");
    Ok(())
}
