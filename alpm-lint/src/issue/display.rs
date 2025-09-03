//! Generic representation of human readable lint issue messages.
//!
//! Contains the [LintIssueDisplay] type, which is the uniform way of formatting issue messages.

use std::{collections::BTreeMap, fmt};

use colored::Colorize;

use crate::Level;

/// A generic structure that represents all possible components of a lint issue display.
///
/// The actual layouting is done in the `Display` implementation of LintIssueDisplay.
///
/// # Visual Layout
///
/// ```text
///    level[lint_rule]: summary      <- header with optional summary
///    --> arrow_line                 <- arrow line with context (optional)
///     |
///     | message                     <- main issue description
///     |
///    help: help_text line 1         <- help section
///          help_text line 2...
///       = custom_link: url          <- custom links (optional)
///       = see: documentation_url    <- auto-generated doc link
/// ```
///
/// # Examples
///
/// ```text
/// warning[source_info::duplicate_architecture]
///   -->  in field 'arch' for package 'example'
///    |
///    | found duplicate value: x86_64
///    |
/// help: Architecture lists should be unique.
///    = see: https://alpm.archlinux.page/lints/...
/// ```
#[derive(Clone, Debug)]
// Allow missing docs, as the individual fields are better explained via the graphic on the struct.
#[allow(missing_docs)]
pub struct LintIssueDisplay {
    pub level: Level,
    pub lint_rule: String,
    pub summary: Option<String>,
    pub arrow_line: Option<String>,
    pub message: String,
    pub help_text: String,
    pub custom_links: BTreeMap<String, String>,
}

impl fmt::Display for LintIssueDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Header with level and lint rule
        let level_str = match self.level {
            Level::Error => "error".bold().red(),
            Level::Deny => "denied".bold().red(),
            Level::Warn => "warning".bold().yellow(),
            Level::Suggest => "suggestion".bold().bright_blue(),
        };

        // Header line
        write!(f, "{}[{}]", level_str, self.lint_rule.blue().bold())?;
        // Optionally append summary to header line or add a newline.
        if let Some(summary) = &self.summary {
            writeln!(f, ": {}", summary.bright_white())?;
        } else {
            writeln!(f)?;
        }

        // Optional context
        if let Some(arrow_line) = &self.arrow_line {
            writeln!(f, "  {} {}", "-->".bright_blue().bold(), arrow_line)?;
        }

        // Start the pipe section.
        // A top and bottom pipe are added for better visual differentiation.
        writeln!(f, "   {}", "|".bright_blue().bold())?;
        for line in self.message.lines() {
            writeln!(f, "   {} {}", "|".bright_blue().bold(), line)?;
        }
        writeln!(f, "   {}", "|".bright_blue().bold())?;

        let mut is_first_line = true;
        for line in self.help_text.lines() {
            // Prefix the very first line with a `help: `.
            if is_first_line {
                writeln!(f, "help: {}", line.bright_white())?;
                is_first_line = false;
                continue;
            }

            // Don't indent empty lines
            if line.is_empty() {
                writeln!(f)?;
            } else {
                writeln!(f, "      {}", line.bright_white())?;
            }
        }

        fn write_link(f: &mut fmt::Formatter<'_>, name: &str, url: &str) -> fmt::Result {
            writeln!(f, "   = {}: {}", name.cyan(), url.underline())
        }

        // Add custom links
        for (name, url) in &self.custom_links {
            write_link(f, name, url)?;
        }

        // Auto-generated documentation URL
        let doc_url = &format!(
            "https://alpm.archlinux.page/lints/index.html#{}",
            self.lint_rule
        );
        write_link(f, "see", doc_url)?;

        Ok(())
    }
}
