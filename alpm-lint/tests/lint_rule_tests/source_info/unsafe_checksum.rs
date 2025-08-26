use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::unsafe_checksum::UnsafeChecksum,
};
use alpm_srcinfo::SourceInfo;
use alpm_types::{SkippableChecksum, Source, digests::Md5};

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

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnsafeChecksum::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::unsafe_checksum");
    Ok(())
}
