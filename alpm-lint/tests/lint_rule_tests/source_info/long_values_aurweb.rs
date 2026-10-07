#![expect(clippy::indexing_slicing, clippy::expect_used)]

use std::str::FromStr;

use alpm_lint::{
    Resources,
    issue::SourceInfoIssue,
    lint_rules::source_info::long_values_aurweb::{LongValuesAurweb, limits},
};
use alpm_lint_config::LintRuleConfiguration;
use alpm_srcinfo::{SourceInfo, source_info::v1::package::Override};
use alpm_types::{Name, PackageDescription, Url};
use testresult::TestResult;

use crate::fixtures::default_source_info_v1;

/// Ensures, that SRCINFO values not exceeding the length limits do not trigger a warning.
#[test]
fn long_values_passes() -> TestResult {
    let source_info = default_source_info_v1()?;

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = LongValuesAurweb::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

/// Ensures, that when SRCINFO fields such as pkgbase, pkgname, pkgdesc or url are too long, a lint
/// is triggered.
#[test]
fn long_values_fails() -> TestResult {
    let mut source_info = default_source_info_v1()?;

    let names = (
        Name::new("a".repeat(limits::PKGBASE + 1).as_str())?,
        Name::new("b".repeat(limits::PKGNAME + 1).as_str())?,
    );
    let descriptions = (
        PackageDescription::new("c".repeat(limits::PKGDESC + 1).as_str()),
        PackageDescription::new("d".repeat(limits::PKGDESC + 1).as_str()),
    );
    let urls = (
        Url::from_str(
            &("https://archlinux.org/".to_string() + "e".repeat(limits::URL + 1).as_str()),
        )?,
        Url::from_str(
            &("https://archlinux.org/".to_string() + "f".repeat(limits::URL + 1).as_str()),
        )?,
    );

    source_info.base.name = names.0.clone();
    source_info.base.description = Some(descriptions.0.clone());
    source_info.base.url = Some(urls.0.clone());

    let package = source_info
        .packages
        .get_mut(0)
        .expect("A package should exist in the test data");
    package.name = names.1.clone();
    package.description = Override::Yes {
        value: descriptions.1.clone(),
    };
    package.url = Override::Yes {
        value: urls.1.clone(),
    };

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = LongValuesAurweb::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert_eq!(
        issues.len(),
        6,
        "6 lint errors should've been found. Instead found: {}",
        issues.len()
    );

    for issue in &issues {
        assert_eq!(issue.lint_rule, lint_rule.scoped_name());
    }

    assert_eq!(
        issues[0].issue_type,
        SourceInfoIssue::BaseField {
            field_name: "pkgbase".into(),
            value: names.0.to_string(),
            context: format!("`pkgbase` value exceeded {} bytes", limits::PKGBASE),
            architecture: None,
        }
        .into()
    );

    assert_eq!(
        issues[1].issue_type,
        SourceInfoIssue::BaseField {
            field_name: "pkgdesc".into(),
            value: descriptions.0.to_string(),
            context: format!("`pkgdesc` value exceeded {} bytes", limits::PKGDESC),
            architecture: None,
        }
        .into()
    );

    assert_eq!(
        issues[2].issue_type,
        SourceInfoIssue::BaseField {
            field_name: "url".into(),
            value: urls.0.to_string(),
            context: format!("`url` value exceeded {} bytes", limits::URL),
            architecture: None,
        }
        .into()
    );

    assert_eq!(
        issues[3].issue_type,
        SourceInfoIssue::PackageField {
            field_name: "pkgname".into(),
            package_name: names.1.to_string(),
            value: names.1.to_string(),
            context: format!("`pkgname` value exceeded {} bytes", limits::PKGBASE),
            architecture: None,
        }
        .into()
    );

    assert_eq!(
        issues[4].issue_type,
        SourceInfoIssue::PackageField {
            field_name: "pkgdesc".into(),
            package_name: names.1.to_string(),
            value: descriptions.1.to_string(),
            context: format!("`pkgdesc` value exceeded {} bytes", limits::PKGDESC),
            architecture: None,
        }
        .into()
    );

    assert_eq!(
        issues[5].issue_type,
        SourceInfoIssue::PackageField {
            field_name: "url".into(),
            package_name: names.1.to_string(),
            value: urls.1.to_string(),
            context: format!("`url` value exceeded {} bytes", limits::URL),
            architecture: None,
        }
        .into()
    );

    Ok(())
}
