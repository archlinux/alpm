use std::str::FromStr;

use alpm_lint::{
    Resources,
    config::LintRuleConfiguration,
    lint_rules::source_info::openpgp_key_id::OpenPGPKeyId,
};
use alpm_srcinfo::SourceInfo;
use alpm_types::OpenPGPIdentifier;

use crate::fixtures::default_source_info_v1;

#[test]
fn openpgp_key_id_passes() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.pgp_fingerprints = vec![OpenPGPIdentifier::from_str(
        "4A0C4DFFC02E1A7ED969ED231C2358A25A10D94E",
    )?];

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = OpenPGPKeyId::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert_eq!(issues.len(), 0);
    Ok(())
}

#[test]
fn openpgp_key_id_fails() -> testresult::TestResult {
    let mut source_info = default_source_info_v1()?;
    source_info.base.pgp_fingerprints = vec![OpenPGPIdentifier::from_str("2F2670AC164DB36F")?];

    let resources = Resources::SourceInfo(SourceInfo::V1(source_info));
    let config = LintRuleConfiguration::default();
    let lint_rule = OpenPGPKeyId::new_boxed(&config);
    let mut issues = Vec::new();

    lint_rule.run(&resources, &mut issues)?;

    assert!(!issues.is_empty(), "A lint error should've been found.");
    assert_eq!(issues[0].lint_rule, "source_info::openpgp_key_id");
    Ok(())
}
