//! Generic representation of a lint issue.

use std::collections::BTreeMap;

use alpm_types::Architecture;
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

use crate::{Level, LintRule, LintScope};

/// An issue a [`LintRule`] may encounter.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LintIssue {
    /// The name of the lint rule that triggers this error.
    pub lint_rule: String,
    /// The severity level of this issue.
    pub level: Level,
    /// The help text that is displayed when the issue is encountered.
    pub help_text: String,
    /// The scope in which the lint is discovered.
    pub scope: LintScope,
    /// The type of issue that is encountered.
    pub issue_type: LintIssueType,
    /// Links that can be appended to an issue.
    /// Stored as a map of `name -> URL`.
    pub links: BTreeMap<String, String>,
}

impl LintIssue {
    /// Creates a new [`LintIssue`] from a [`LintRule`] and [`LintIssueType`].
    pub fn from_rule<T: LintRule>(rule: &T, issue_type: LintIssueType) -> Self {
        LintIssue {
            lint_rule: rule.scoped_name(),
            level: rule.level(),
            help_text: rule.help_text(),
            scope: rule.scope(),
            issue_type,
            links: rule.extra_links().unwrap_or_default(),
        }
    }
}

/// The type of issue that may be encountered during linting.
///
/// This is used to categorize lint issues and to provide detailed data
/// for good error messages for each type of issue.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LintIssueType {
    /// All issues that can be encountered when linting a [SRCINFO] file.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    SourceInfo(SourceInfoIssue),
}

/// A specific type of [SRCINFO] related lint issues that may be encountered during linting.
///
/// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourceInfoIssue {
    /// A generic issue that only consists of some text without any additional fields.
    ///
    /// Use this for one-off issues that don't fit any other "issue category".
    /// The lint rule must take care of the formatting itself.
    ///
    /// # Note
    ///
    /// If you find yourself using this variant multiple times in a similar manner, consider
    /// creating a dedicated variant for that use case.
    Generic {
        /// A brief, one-line summary of the issue for display above the main error line.
        ///
        /// This is used to populate [`LintIssueDisplay::summary`].
        summary: String,

        /// Additional context that can be displayed between summary and message.
        ///
        /// This is used to populate [`LintIssueDisplay::arrow_line`].
        arrow_line: Option<String>,

        /// The detailed message describing this issue, shown in the context section.
        ///
        /// This can contain more specific information about what was found and where.
        ///
        /// This is used to populate [`LintIssueDisplay::message`].
        message: String,
    },

    /// A lint issue on a `PackageBase` field.
    BaseField {
        /// The field name which causes the issue.
        ///
        /// Used as [`LintIssueDisplay::arrow_line`] in the form of:
        /// `in field {field_name}`
        field_name: String,

        /// The value that causes the issue.
        ///
        /// Used as [`LintIssueDisplay::message`] in the form of:
        /// `"{context}: {value}"`
        value: String,

        /// Additional context that describes what kind of issue is found.
        ///
        /// Used as [`LintIssueDisplay::message`] in the form of:
        /// `"{context}: {value}"`
        context: String,

        /// The architecture in case the field is architecture specific.
        ///
        /// If this is set, it'll be used as [`LintIssueDisplay::message`] in the form of:
        /// `"{context}: {value} for architecture {arch}"`
        architecture: Option<Architecture>,
    },

    /// A lint issue on a field that belongs to a specific package.
    PackageField {
        /// The field name which causes the issue.
        ///
        /// Used as [`LintIssueDisplay::arrow_line`] in the form of:
        /// `format!("in field {field_name} for package {package_name}")`
        field_name: String,

        /// The name of the package for which the issue is detected.
        ///
        /// Used as [`LintIssueDisplay::arrow_line`] in the form of:
        /// `"in field {field_name} for package {package_name}"`
        package_name: String,

        /// The value that causes the issue.
        ///
        /// Used as [`LintIssueDisplay::message`] in the form of:
        /// `"{context}: {value}"`
        value: String,

        /// Additional context that describes what kind of issue is found.
        ///
        /// Used as [`LintIssueDisplay::message`] in the form of:
        /// `"{context}: {value}"`
        context: String,

        /// The architecture in case the field is architecture specific.
        ///
        /// If this is set, it'll be used as [`LintIssueDisplay::message`] in the form of:
        /// `"{context}: {value} for architecture {arch}"`
        architecture: Option<Architecture>,
    },

    /// A required field is missing from the package base.
    MissingField {
        /// The name of the field that is missing.
        field_name: String,
    },
}

impl SourceInfoIssue {
    /// Takes a field name with an optional architecture and returns the correct
    /// [SRCINFO] formatting as bold text.
    ///
    /// [SRCINFO]: https://alpm.archlinux.page/specifications/SRCINFO.5.html
    pub fn field_fmt(field_name: &str, architecture: Option<Architecture>) -> ColoredString {
        match architecture {
            Some(arch) => format!("{field_name}_{arch}").bold(),
            None => field_name.bold(),
        }
    }
}

impl From<SourceInfoIssue> for LintIssueType {
    fn from(issue: SourceInfoIssue) -> Self {
        LintIssueType::SourceInfo(issue)
    }
}
