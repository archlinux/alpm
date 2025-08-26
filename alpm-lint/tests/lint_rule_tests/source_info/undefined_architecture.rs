use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::undefined_architecture::UndefinedArchitecture,
};
use alpm_srcinfo::{
    SourceInfo,
    source_info::v1::{package::PackageArchitecture, package_base::PackageBaseArchitecture},
};
use alpm_types::Architecture;

use crate::fixtures::default_source_info_v1;

#[test]
fn undefined_architecture_base_passes() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = vec![Architecture::X86_64, Architecture::Aarch64];

    // Add base architecture properties only for declared architectures
    let arch_properties = PackageBaseArchitecture::default();
    source_info
        .base
        .architecture_properties
        .insert(Architecture::X86_64, arch_properties.clone());
    source_info
        .base
        .architecture_properties
        .insert(Architecture::Aarch64, arch_properties);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UndefinedArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

#[test]
fn undefined_architecture_base_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = vec![Architecture::X86_64]; // Only declare x86_64

    // Add base architecture properties for undeclared architecture
    let arch_properties = PackageBaseArchitecture::default();
    source_info
        .base
        .architecture_properties
        .insert(Architecture::Aarch64, arch_properties); // Use aarch64 (undeclared)

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UndefinedArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should have been found");
    assert_eq!(issues[0].lint_rule, "source_info::undefined_architecture");
    Ok(())
}

#[test]
fn undefined_architecture_package_passes() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = vec![Architecture::X86_64, Architecture::Aarch64];

    // Add package architecture properties only for declared architectures
    let arch_properties = PackageArchitecture::default();
    source_info.packages[0]
        .architecture_properties
        .insert(Architecture::X86_64, arch_properties.clone());
    source_info.packages[0]
        .architecture_properties
        .insert(Architecture::Aarch64, arch_properties);

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UndefinedArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(issues.is_empty(), "No lint issues should have been found");
    Ok(())
}

#[test]
fn undefined_architecture_package_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.architectures = vec![Architecture::X86_64]; // Only declare x86_64

    // Add package architecture properties for undeclared architecture
    let arch_properties = PackageArchitecture::default();
    source_info.packages[0]
        .architecture_properties
        .insert(Architecture::Aarch64, arch_properties); // Use aarch64 (undeclared)

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = UndefinedArchitecture::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should have been found");
    assert_eq!(issues[0].lint_rule, "source_info::undefined_architecture");
    Ok(())
}
