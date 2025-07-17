//! Generic representation of a lint issue.

use serde::{Deserialize, Serialize};

use crate::{Level, LintScope};

/// A lint failed in some kind of way.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LintIssue {
    /// The name of the lint rule that produced this error
    pub lint_rule: String,
    /// The severity level of this issue
    pub level: Level,
    /// The help text that will be displayed when this lint is encountered.
    pub help_text: String,
    /// The
    pub issue_type: LintIssueType,
}

/// Various types of lint issue may be encountered during linting.
///
/// This enum reflects these types in a generic fashion.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LintIssueType {
    /// A lint error is encountered on a single field.
    Field {
        /// The scope on which the lint was discovered.
        scope: LintScope,
    },
}
