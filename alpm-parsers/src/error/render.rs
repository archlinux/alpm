//! Rendering logic for [`ParseStack`] errors.

use std::fmt;

use colored::Colorize;
use unicode_width::UnicodeWidthChar;

use super::parse_stack::ParseStack;
use crate::error::layer::LayerRef;

/// Writes a potentially multi-line footer message for a parser layer.
fn write_footer_message(out: &mut String, guide: &str, message: &str) {
    let mut lines = message.lines();
    let Some(first) = lines.next() else {
        return;
    };

    out.push_str(&format!("{}→ {}\n", guide, first.dimmed()));
    for line in lines {
        out.push_str(&format!("{}  {}\n", guide, line.dimmed()));
    }
}

/// Returns the start and end byte offsets of the line containing `at`.
///
/// This function helps us navigate the given document.
/// We usually start somewhere in the middle of the input, without any knowledge of what's around
/// the current span/pointer.
fn line_bounds(src: &str, at: usize) -> (usize, usize) {
    let at = src.floor_char_boundary(at);
    let line_start = src[..at].rfind('\n').map_or(0, |i| i + 1);
    let line_end = src[at..].find('\n').map_or(src.len(), |i| at + i);
    (line_start, line_end)
}

/// Returns the indentation and the underline for the span between `span_start` and `at`.
///
/// The two returned strings are meant to be concatenated and are followed by the caret that
/// points at the character, which caused the error:
///
/// ```text
/// 1 |    foo-bar
///   |    ~~~~~~^
///     ^^^ indentation
///        ^^^^^^ underline
/// ```
///
/// There are two scenarios that we need to handle to achieve a correct underline and to place the
/// caret pointer at the correct position:
///
/// - Multi-width UTF-8 graphemes. To handle this, we measure the display width of each character on
///   the error line, and insert the correct amount of spaces/`~`.
///
/// - Tabs cannot be handled 100% correctly, as their rendered width depends on the terminal's tab
///   stops. We work around the issue as follows:
///   - When encountering a tab while building the indentation, we simply place a `\t` at the same
///     position as in the string above.
///   - When encountering a tab while building the underline, we insert a literal `\t` there as
///     well. This visually breaks the underline, but as there's no way to determine its display
///     width, we accept the compromise of a broken indicator line in favor of a correct offset for
///     the pointer.
fn underline(src: &str, line_start: usize, span_start: usize, at: usize) -> (String, String) {
    // Clamped all chars down to the closest valid UTF-8 character boundary, in case the parsing
    // error points into the middle of a multi-byte character.
    let line_start = src.floor_char_boundary(line_start);
    let span_start = src.floor_char_boundary(span_start).max(line_start);
    let at = src.floor_char_boundary(at).max(span_start);

    // Build the indentation string.
    let mut indent = String::new();
    for char in src[line_start..span_start].chars() {
        match char {
            '\t' => indent.push('\t'),
            _ => indent.push_str(&" ".repeat(char.width().unwrap_or(0))),
        }
    }

    // Build the underline.
    let mut span = String::new();
    for char in src[span_start..at].chars() {
        match char {
            '\t' => span.push('\t'),
            _ => span.push_str(&"~".repeat(char.width().unwrap_or(0))),
        }
    }

    (indent, span)
}

/// Formats the given expected literals into a single "expected" message:
///
/// ```text
/// expected `]`
/// expected one of: `]`, `}`
/// ```
///
/// Returns `None` if the provided literal list is empty.
fn expected_message(literals: &[String]) -> Option<String> {
    match literals {
        [] => None,
        [literal] => Some(format!("expected {literal}")),
        literals => Some(format!("expected one of: {}", literals.join(", "))),
    }
}

/// The maximum snippet length before showing a `…` indicator.
const SNIPPET_SIZE: usize = 20;

/// Returns a preview of the input starting at `from`, truncated to [`SNIPPET_SIZE`] chars or the
/// next newline.
fn snippet_at(src: &str, from: usize) -> String {
    let from = src.floor_char_boundary(from);
    let (_, line_end) = line_bounds(src, from);
    let raw = &src[from..line_end];
    let shown: String = raw.chars().take(SNIPPET_SIZE).collect();

    // If there's trailing content on that line, also show a `…` indicator.
    if raw.chars().count() > SNIPPET_SIZE {
        format!("{shown}…")
    } else {
        shown
    }
}

impl fmt::Display for ParseStack<'_> {
    /// Displays this parse error.
    ///
    /// The rendered output is structured into three visual sections:
    ///
    /// 1. A headline with the innermost available label context.
    /// 2. A source snippet with an underline spanning from beginning of the innermost named layer
    ///    up to the exact failing character. Any expected literals are shown next to the caret.
    /// 3. A footer that provides error context from outermost to innermost layers.
    ///
    /// Named layers are rendered with their parser name and a mini source preview:
    ///
    /// ```text
    /// error: invalid package release
    ///   |
    /// 1 | foo-1:1.0.0-bar-any
    ///   |             ^ expected positive decimal integer
    ///   |
    ///   = while parsing:
    ///     installed package name: (foo-1:1.0.0-bar-any)
    ///     └ alpm-package-version: (1:1.0.0-bar-any)
    ///       │ → an alpm-package-version (full or full with epoch) followed by a `-` and an alpm-architecture
    ///       └ alpm-pkgrel: (bar-any)
    ///         → invalid package release
    ///         → A freeform description over here
    ///         → expected positive decimal integer
    /// ```
    ///
    /// Pending context that has not yet unwound past a named layer is rendered as an anonymous
    /// outermost layer:
    ///
    /// ```text
    ///   = while parsing:
    ///     → alpm-package file name
    ///     → a package name, followed by an alpm-package-version...
    ///     └ installed package name: (foo-1:1.0.0_any)
    /// ```
    ///
    /// Color output is controlled globally via [`colored::control`] (for example via
    /// [`colored::control::set_override`]).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let src = self.source;
        let at = src.floor_char_boundary(self.at);

        // We locate the failing line using the byte offsets (as that's what winnow provides).
        // To support Unicode however, we consider the width of the unicode characters, so that
        // the positioning of the underline characters stays correct.
        let (line_start, line_end) = line_bounds(src, at);
        let line_number = src[..at].bytes().filter(|&b| b == b'\n').count() + 1;
        let line = &src[line_start..line_end];

        // Get the neighboring lines around the line with the failure.
        // Empty neighbors are dropped, as they provide no additional context.
        //
        // Everything before `line_start` ends with a newline, so the last line of that slice is
        // the previous line. Everything from `line_end` starts with the current line's newline,
        // so the second line of that slice is the next line.
        let prev = src[..line_start]
            .lines()
            .next_back()
            .map(|text| (line_number - 1, text));
        let next = src[line_end..]
            .lines()
            .nth(1)
            .map(|text| (line_number + 1, text));

        // Calculate the width of the largest line number.
        // We have to make sure that the padding is equal across all codeblock lines.
        let (max_number_width, number_padding, line_number) = if prev.is_none() && next.is_none() {
            // Special case where we're handling a single-line input, in which case we just scrap
            // the number altogether.
            (0, "".to_string(), None)
        } else {
            let max_number = next.map_or(line_number, |(n, _)| n);
            let max_number_width = max_number.to_string().len();
            (
                max_number_width,
                " ".repeat(max_number_width),
                Some(line_number),
            )
        };

        // Mini helper closure to write indent lines witih the given width.
        // ```
        //    |
        // 9  |
        // 10 |
        //    |
        // ```
        let codeblock_line = |number: Option<usize>, line: &str| {
            let number = number
                .map(|n| format!("{n:>max_number_width$}"))
                .unwrap_or_else(|| number_padding.to_string());
            format!("{} {} {}\n", number.blue(), "|".blue(), line)
        };

        let mut out = String::new();

        // Header.
        // E.g. `error: invalid input`
        out += &format!(
            "{} {}\n",
            "error:".red().bold(),
            format!("invalid {}", self.headline()).bold(),
        );

        // Source code block with error context.
        // - Spacer line
        // - optional previous line
        // - failing line
        // - span underline with optional "expected" messages
        // - optional next line
        // - optional spacer line

        // Add an empty line with some padding.
        out += &codeblock_line(None, "");

        // Print the previous line
        if let Some((line_number, text)) = prev
            && !text.is_empty()
        {
            out += &codeblock_line(Some(line_number), text);
        }

        // Print the line with the actual error.
        out += &codeblock_line(line_number, line);

        // Build the underline from the innermost named layer's start to the failing byte.
        // If no named layer exists, this falls back to the failure position.
        let (indent, span) = underline(src, line_start, self.innermost().start(), at);

        // Print the span underline, with the first line of the "expected" message next to the
        // caret. The "expected" message lists all expected literals of the innermost layer.
        let mut span_line = format!("{}{}{}", indent, span.red(), "^".red().bold());
        let expected = expected_message(&self.innermost_expected()).unwrap_or_default();
        let mut expected_lines = expected.lines();
        if let Some(first) = expected_lines.next() {
            span_line.push_str(&format!(" {}", first.red()));
        }
        out += &codeblock_line(None, &span_line);

        // Print all follow-up lines of the "expected" message.
        // The text is indented past the caret, so it aligns with the first "expected" line.
        // The underline only consists of single width characters, hence the char count is the
        // width we have to pad. The `+1` is the caret itself.
        let continuation = format!("{indent}{}", " ".repeat(span.chars().count() + 1));
        for line in expected_lines {
            out += &codeblock_line(None, &format!("{continuation} {}", line.red()));
        }

        // Print the next line after the error
        if let Some((line_number, text)) = next
            && !text.is_empty()
        {
            out += &codeblock_line(Some(line_number), text);
        }

        // The footer section should only be drawn if it provides new information.
        //
        // Two or more layers are a safe draw, as there will always be new information.
        //
        // To draw in case of one layer, there must be either
        // - Staged content
        // - Or at least a label context on that layer
        //
        // Descriptions are only ever rendered in the footer. Hence, the footer must always be
        // drawn if any layer, including the anonymous one for pending context, has a description.
        //
        // In all other cases, the footer provides no additional information.
        let has_descriptions = self
            .layer_stack()
            .iter()
            .any(|layer| layer.descriptions().is_some());
        let has_labeled_layer = self
            .layers
            .first()
            .is_some_and(|layer| LayerRef::Named(layer).label_message().is_some());

        let should_draw_footer_section =
            // 2+ layers
            self.layers.len() > 1
            // 1 layer + pending context
            || (!self.layers.is_empty() && !self.pending.is_empty())
            // 1 layer with a label
            || has_labeled_layer
            // Any layer with a description
            || has_descriptions
            // There's an external error
            || self.external.is_some();

        // Only show the line below as visual buffer, if there's some content.
        if should_draw_footer_section {
            out += &codeblock_line(None, "");
        }

        // The footer section.
        //
        // Displays the layer stack, from outermost to innermost.
        if should_draw_footer_section {
            out += &format!(
                "{} {} {}\n",
                number_padding,
                "=".blue(),
                "while parsing:".dimmed(),
            );

            let mut layers_iter = self.layer_stack().into_iter().enumerate().peekable();
            while let Some((depth, layer)) = layers_iter.next() {
                if let Some(name) = layer.name() {
                    // The prefix for the layer's name
                    let branch = if depth == 0 {
                        String::new()
                    } else {
                        format!("{}└ ", " ".repeat((depth - 1) * 2))
                    };

                    let snippet = snippet_at(src, layer.start());
                    out += &format!(
                        "{}   {}{}: {}\n",
                        number_padding,
                        branch,
                        name.bold(),
                        format!("({snippet})").dimmed(),
                    );
                }

                // Determine the indentation amount for the current layer.
                let nested_padding = number_padding.len() + 3 + 2 * depth;

                // In case we are not on the last/innermost layer, add a UTF-8 border as a
                // visual guide.
                let guide = if layers_iter.peek().is_none() {
                    " ".repeat(nested_padding)
                } else {
                    format!("{}│ ", " ".repeat(nested_padding))
                };

                // Print the label of the layer.
                let label = layer.label_message();
                if let Some(label) = label {
                    write_footer_message(&mut out, &guide, &format!("invalid {label}"));
                }

                // Print the free-form descriptions.
                if let Some(descriptions) = layer.descriptions() {
                    write_footer_message(&mut out, &guide, &descriptions);
                }

                // Print information about all expected literals at this position.
                if let Some(message) = expected_message(&layer.expected_literals()) {
                    write_footer_message(&mut out, &guide, &message);
                }
            }
        }

        // This is some extra handling in case an error came from an external error,
        // such as a `try_map`.
        if let Some(external) = &self.external {
            out += &format!(
                "{} {} {}\n",
                number_padding,
                "=".blue(),
                format!("an error occurred: {external}").dimmed(),
            );
        }
        f.write_str(&out)
    }
}
