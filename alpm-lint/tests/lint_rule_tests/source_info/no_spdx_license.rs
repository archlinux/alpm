use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::no_spdx_license::NotSPDX,
};
use alpm_srcinfo::SourceInfo;
use alpm_types::License;

use crate::fixtures::default_source_info_v1;

#[test]
fn no_spdx_license_passes() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.licenses = vec![License::from_str("Apache-2.0")?];

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = NotSPDX::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

#[test]
fn no_spdx_license_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.licenses = vec![License::from_str("Apache")?];

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = NotSPDX::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::no_spdx_license");
    Ok(())
}
