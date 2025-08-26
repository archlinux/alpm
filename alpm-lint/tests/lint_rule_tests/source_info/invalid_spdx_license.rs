use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::invalid_spdx_license::NotSPDX,
};
use alpm_srcinfo::{
    SourceInfo,
    source_info::v1::package::{Override, Package},
};
use alpm_types::{License, Name};

use crate::fixtures::default_source_info_v1;

#[test]
fn invalid_spdx_license_passes() -> testresult::TestResult {
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
fn package_base_invalid_spdx_license_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.licenses = vec![License::from_str("Apache")?];

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = NotSPDX::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::invalid_spdx_license");
    Ok(())
}

#[test]
fn package_invalid_spdx_license_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    let mut package = Package::from(Name::from_str("test-package")?);
    package.licenses = Override::Yes {
        value: vec![License::from_str("Apache")?],
    };

    source_info.packages.push(package);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = NotSPDX::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::invalid_spdx_license");
    Ok(())
}
