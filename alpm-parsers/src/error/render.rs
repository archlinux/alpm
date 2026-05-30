//! Rendering logic for [`ParseStack`] errors.

use std::fmt;

use colored::Colorize;
use unicode_width::UnicodeWidthChar;

use super::parse_stack::ParseStack;
use crate::error::layer::LayerRef;

/// Write a potentially multi-line "expected" string in the span underline section of the parser
/// error.
fn write_inline_expected<F>(
    out: &mut String,
    codeblock_line: &F,
    span_line: &str,
    span_line_end: usize,
    expected: &str,
) where
    F: Fn(Option<usize>, &str) -> String,
{
    // If there are no "expected" style messages, only print the span line.
    let mut lines = expected.lines();
    let Some(first) = lines.next() else {
        out.push_str(&codeblock_line(None, span_line));
        return;
    };

    // Print the span line + first line of "expected" string.
    out.push_str(&codeblock_line(
        None,
        &format!("{} {}", span_line, format!("expected {first}").red(),),
    ));

    // Print all potential follow-up "expected" lines.
    // The text is intended past the span line.
    let continuation = " ".repeat(span_line_end + 1);
    for line in lines {
        out.push_str(&codeblock_line(
            None,
            &format!("{continuation} {}", line.red(),),
        ));
    }
}

/// Write a potentially multi-line footer message for a parser layer.
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

/// Clamp `index` down to the closest valid UTF-8 character boundary.
///
/// This **may** be necessary in case the parsing error points to some broken UTF-8 character, which
/// would blow up any string handling.
fn floor_char_boundary(src: &str, index: usize) -> usize {
    let mut index = index.min(src.len());
    while index > 0 && !src.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Return the start and end byte offsets of the line containing `at`.
///
/// This helper function helps us navigate the given document.
/// We usually start somewhere in the middle of the input, without any knowledge what's around the
/// current span/pointer.
fn line_bounds(src: &str, at: usize) -> (usize, usize) {
    let at = floor_char_boundary(src, at);
    let line_start = src[..at].rfind('\n').map_or(0, |i| i + 1);
    let line_end = src[at..].find('\n').map_or(src.len(), |i| at + i);
    (line_start, line_end)
}

/// Return the display width between `line_start` and `index`.
fn width_till(src: &str, line_start: usize, index: usize) -> usize {
    let line_start = floor_char_boundary(src, line_start);
    let index = floor_char_boundary(src, index).max(line_start);

    // Calculate the actual display width of the slice.
    src[line_start..index]
        .chars()
        .map(|ch| ch.width().unwrap_or(0))
        .sum()
}

const SNIPPET_SIZE: usize = 20;

/// Preview of the input starting at `from`, truncated to [`SNIPPET_SIZE`] chars or the next
/// newline.
fn snippet_at(src: &str, from: usize) -> String {
    let from = floor_char_boundary(src, from);
    let (_, line_end) = line_bounds(src, from);
    let raw = &src[from..line_end];
    let shown: String = raw.chars().take(SNIPPET_SIZE).collect();
    if raw.chars().count() > SNIPPET_SIZE {
        format!("{shown}…")
    } else {
        shown
    }
}

impl fmt::Display for ParseStack<'_> {
    /// Display this parse error.
    ///
    /// The rendered output is structured into three visual sections:
    ///
    /// 1. A headline with the innermost available [`StrContext::Label`](winnow::error::StrContext).
    /// 2. A source snippet with an underline spanning from beginning of the innermots named layer
    ///    up to the exact failing character.
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
    ///         → expected positive decimal integer
    /// ```
    ///
    /// Pending context that has not yet unwound past a named layer is rendered as an anonymous
    /// outermost layer:
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
        let at = floor_char_boundary(src, self.at);

        // We locate the failing line using the byte offsets (as that's what winnow provides).
        // To support Unicode however, we consider the width of the unicode characters, so that
        // the positioning of the underline characters stays correct.
        let (line_start, line_end) = line_bounds(src, at);
        let line_number = src[..at].bytes().filter(|&b| b == b'\n').count() + 1;
        let line = &src[line_start..line_end];

        // The width of the underline: from the innermost named layer's start to the
        // failing byte. If no named layer exists, fall back to the failure position.
        let span_start = width_till(src, line_start, self.innermost().start().max(line_start));
        let span_end = width_till(src, line_start, at);

        // Get the line before the line with the failure.
        // If the start pointer isn't at the very start of the input, there must be another line
        // above.
        let prev = if line_start > 0 {
            let nl = line_start - 1;
            // Find the next previous newline. If there is none, use the start of input.
            let start = src[..nl].rfind('\n').map_or(0, |i| i + 1);
            let text = &src[start..nl];
            (!text.is_empty()).then_some((line_number - 1, text))
        } else {
            None
        };

        // Get the line after the line with the failure.
        let next = if line_end < src.len() {
            let start = line_end + 1;
            // Seek either to the next newline or, if there is none, to the EOF.
            let end = src[start..].find('\n').map_or(src.len(), |i| start + i);
            let text = &src[start..end];
            (!text.is_empty()).then_some((line_number + 1, text))
        } else {
            None
        };

        // Calculate the width of the largest line number.
        // We have to make sure that the padding is equal across all codeblock lines.
        let max_number = next.map_or(line_number, |(n, _)| n);
        let mut max_number_width = max_number.to_string().len();
        let mut number_padding = " ".repeat(max_number_width);
        let mut line_number = Some(line_number);

        // Special case where we're handling with a single-line input.
        // In this case, we just scrap the number altogether.
        if prev.is_none() && next.is_none() {
            max_number_width = 0;
            number_padding = "".to_string();
            line_number = None;
        }

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
        if let Some((line_number, text)) = prev {
            out += &codeblock_line(Some(line_number), text);
        }

        // Print the line with the actual error.
        out += &codeblock_line(line_number, line);

        // Print the span underline with the context error string.
        let mut span_line = String::new();
        span_line.push_str(&" ".repeat(span_start));
        span_line.push_str(&format!(
            "{}{}",
            "~".repeat(span_end.saturating_sub(span_start)).red(),
            "^".red().bold(),
        ));
        let span_line_end = span_end.saturating_sub(span_start) + 1;
        if let Some(expected) = self.first_expected() {
            write_inline_expected(
                &mut out,
                &codeblock_line,
                &span_line,
                span_line_end,
                &expected,
            );
        } else {
            out += &codeblock_line(None, &span_line);
        }

        // Print the next line after the error
        if let Some((line_number, text)) = next {
            out += &codeblock_line(Some(line_number), text);
        }

        // The footer section should only be drawn if it provides new information.
        //
        // Two or more layers are a safe draw, as there will always be new information.
        //
        // To draw in case of one layer, there must be either
        // - Staged content
        // - Or at least a `StrContext::Label` on that layer
        //
        // Otherwise, the footer provides no additional information.
        let should_draw_footer_section =
            // 2+ layers
            self.layers.len() > 1 ||
            // 1 layer + pending context
            (!self.layers.is_empty() && !self.pending.is_empty()) ||
            // 1 layer with a label
            if let Some(layer) = self.layers.first() {
                LayerRef::Named(layer).label_message().is_some()
            } else {
                false
            }
        ;

        // Only show the line below as visual buffer, if there's some content....
        if should_draw_footer_section || self.external.is_some() {
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

                // In case we are not on the last (innermost) layer, add a utf-8 border as a visual
                // guide.
                let guide = if layers_iter.peek().is_none() {
                    " ".repeat(nested_padding)
                } else {
                    format!("{}│ ", " ".repeat(nested_padding))
                };

                let label = layer.label_message();
                if let Some(label) = label {
                    write_footer_message(&mut out, &guide, &format!("invalid {label}"));
                }
                if let Some(expected) = layer.expected_message() {
                    write_footer_message(&mut out, &guide, &format!("expected {expected}"));
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
                format!("note: {external}").dimmed(),
            );
        }
        f.write_str(&out)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{snippet_at, width_till};

    // Make sure that the `snippet_at` function doesn't exceed newlines.
    #[rstest]
    #[rstest]
    #[case::till_newline(1, "irst")]
    #[case::till_end(6, "second")]
    fn snippet_until_newline(#[case] at: usize, #[case] expected: &str) {
        assert_eq!(snippet_at("first\nsecond", at), expected);
    }

    // Make sure that the `snippet_at` function doesn't exceed newlines.
    #[rstest]
    #[case::ascii(1, 1)]
    #[case::truncated_boundary(2, 1)]
    #[case::multiwidth_emoji(3, 3)]
    fn display_column_supports_unicode(#[case] end: usize, #[case] expected: usize) {
        let input = "a\u{1F642}b";
        assert_eq!(width_till(input, 0, end), expected);
    }
}
