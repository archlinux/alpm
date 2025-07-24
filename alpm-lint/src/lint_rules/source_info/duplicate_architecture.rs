//! An lint that is thrown when an architecture has been specified twice.

use crate::prelude::*;

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

    fn run(&self, _resources: Resources) {}
}
