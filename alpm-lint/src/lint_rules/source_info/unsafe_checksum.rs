//! The file verification for some source in a .SRCINFO file uses an unsafe hash algorithm.

use crate::prelude::*;

/// The file verification for some source in a .SRCINFO file uses an unsafe hash algorithm.
#[derive(Clone, Debug)]
pub struct UnsafeChecksum {}

impl UnsafeChecksum {
    /// Create a new, boxed instance of [`UnsafeChecksum`].
    ///
    /// This is used to register the lint on the `LintStore`.
    pub fn new_boxed(_: &LintRuleConfiguration) -> Box<dyn LintRule> {
        Box::new(Self {})
    }
}

impl LintRule for UnsafeChecksum {
    fn name(&self) -> &'static str {
        "unsafe_checksum"
    }

    fn scope(&self) -> LintScope {
        LintScope::SourceInfo
    }

    fn level(&self) -> Level {
        crate::Level::Deny
    }

    fn groups(&self) -> &'static [LintGroup] {
        &[LintGroup::Pedantic]
    }

    fn help_text(&self) -> &'static str {
        r#"Some checksum algorithms, such as `MD5` or `SHA1`, have been determined as cryptographically unsound.

Using such checksum algorithm to verify downloaded source artifacts is a security risk and should be avoided at all cost.

Duplicate architecture declarations such as `arch=(x86_64 x86_64)` have no effect, as the second occurrence will simply be ignored.
"#
    }

    fn run(&self, _resources: Resources) {}
}
