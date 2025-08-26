use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::duplicate_architecture::DuplicateArchitecture,
};
use alpm_srcinfo::SourceInfo;
use alpm_types::Architecture;
use rstest::rstest;

use crate::fixtures::default_source_info_v1;

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

    assert!(issues.is_empty(), "No lint issues should have been found");
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

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::duplicate_architecture");
    Ok(())
}
