//! Tests for source info scope lint rules.

use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::{
        duplicate_architecture::DuplicateArchitecture,
        unsafe_checksum::UnsafeChecksum,
    },
};
use alpm_srcinfo::SourceInfo;
use alpm_types::{Architecture, SkippableChecksum, Source, digests::Md5};
use rstest::rstest;

mod fixtures;
use fixtures::default_source_info_v1;

#[rstest]
#[case::x86_64_and_aarch64(vec![Architecture::X86_64, Architecture::Aarch64])]
#[case::single_architecture(vec![Architecture::Any])]
fn duplicate_architecture_passes(
    #[case] architectures: Vec<Architecture>,
) -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = architectures;

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = DuplicateArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert_eq!(issues.len(), 0);
    Ok(())
}

#[rstest]
#[case::duplicate_x86_64(vec![Architecture::X86_64, Architecture::X86_64])]
#[case::duplicate_with_others(vec![Architecture::Aarch64, Architecture::X86_64, Architecture::X86_64])]
fn duplicate_architecture_fails(
    #[case] architectures: Vec<Architecture>,
) -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = architectures;

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = DuplicateArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].lint_rule, "source_info::duplicate_architecture");
    Ok(())
}

#[test]
fn unsafe_checksum_passes() -> testresult::TestResult {
    let source_info = default_source_info_v1()?;

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnsafeChecksum::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert_eq!(issues.len(), 0);
    Ok(())
}

#[rstest]
#[case::md5_checksum(
    vec![Source::from_str("https://example.com/source.tar.gz")?], 
    vec![SkippableChecksum::<Md5>::from_str("11111111111111111111111111111111")?]
)]
fn unsafe_checksum_fails(
    #[case] sources: Vec<Source>,
    #[case] md5sums: Vec<SkippableChecksum<Md5>>,
) -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;

    source_info.base.sources = sources;
    source_info.base.md5_checksums = md5sums;

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UnsafeChecksum::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    // Currently not implemented, so expecting 0 for now
    assert_eq!(issues.len(), 0);
    Ok(())
}
