//! An lint that is thrown when an architecture has been specified twice.

use std::collections::HashSet;

use alpm_srcinfo::SourceInfo;

use crate::{internal_prelude::*, issue::LintIssueType};

/// An lint that is thrown when an architecture has been specified twice.
#[derive(Clone, Debug)]
pub struct DuplicateArchitecture {}

impl DuplicateArchitecture {
    /// Create a new, boxed instance of [`DuplicateArchitecture`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(DuplicateArchitecture {})
    }
}

impl LintRule for DuplicateArchitecture {
    fn name(&self) -> &'static str {
        "duplicate_architecture"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn help_text(&self) -> &'static str {
        r#"Architecture lists for packages should always be unique.

Duplicate architecture declarations such as `arch=(x86_64 x86_64)` have no effect, as the second occurrence will simply be ignored.
"#
    }

    fn run(&self, resources: &Resources, issues: &mut Vec<LintIssue>) -> Result<(), Error> {
        // Extract the required resources.
        let source_info = match resources {
            Resources::SourceRepository {
                source_info: SourceInfo::V1(source_info),
                ..
            }
            | Resources::SourceInfo(SourceInfo::V1(source_info)) => source_info,
            _ => {
                return Err(Error::InvalidResources {
                    scope: resources.scope(),
                    lint_rule: self.scoped_name(),
                    expected: LintScope::SourceInfo,
                });
            }
        };

        let mut known = HashSet::new();
        println!("{:?}", source_info.base.architectures);
        for architecture in &source_info.base.architectures {
            if known.contains(&architecture) {
                issues.push(LintIssue {
                    lint_rule: self.scoped_name(),
                    level: self.level(),
                    help_text: self.help_text().to_string(),
                    issue_type: LintIssueType::Field {
                        scope: LintScope::SourceInfo,
                    },
                });
            }
            known.insert(architecture);
        }

        Ok(())
    }
}
