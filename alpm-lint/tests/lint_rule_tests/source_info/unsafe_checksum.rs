use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::unsafe_checksum::UnsafeChecksum,
};
use alpm_srcinfo::{SourceInfo, source_info::v1::package_base::PackageBaseArchitecture};
use alpm_types::{
    SkippableChecksum,
    Source,
    SystemArchitecture,
    digests::{Md5, Sha1},
};

use crate::fixtures::default_source_info_v1;

#[test]
fn unsafe_checksum_passes() -> testresult::TestResult {
    let source_info = default_source_info_v1()?;

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnsafeChecksum::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

#[test]
fn unsafe_checksum_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;

    source_info.base.sources = vec![Source::from_str("https://example.com/source.tar.gz")?];
    source_info.base.md5_checksums = vec![SkippableChecksum::<Md5>::from_str(
        "11111111111111111111111111111111",
    )?];
    source_info.base.sha1_checksums = vec![SkippableChecksum::<Sha1>::from_str(
        "1111111111111111111111111111111111111111",
    )?];

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnsafeChecksum::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::unsafe_checksum");
    assert_eq!(issues[1].lint_rule, "source_info::unsafe_checksum");
    Ok(())
}

#[test]
fn architecture_specific_unsafe_checksum_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;

    let architecture = PackageBaseArchitecture {
        sources: vec![Source::from_str("https://example.com/source.tar.gz")?],
        md5_checksums: vec![SkippableChecksum::<Md5>::from_str(
            "11111111111111111111111111111111",
        )?],
        sha1_checksums: vec![SkippableChecksum::<Sha1>::from_str(
            "1111111111111111111111111111111111111111",
        )?],
        ..PackageBaseArchitecture::default()
    };

    source_info
        .base
        .architecture_properties
        .insert(SystemArchitecture::X86_64, architecture);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnsafeChecksum::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::unsafe_checksum");
    assert_eq!(issues[1].lint_rule, "source_info::unsafe_checksum");
    Ok(())
}
