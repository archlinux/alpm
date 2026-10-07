//! Integration tests for our own error type.
//!
//! This modules declares parsers for an "imaginary" document format.
//! This format is then used to validate our error format output.
//!
//! # Format
//!
//! At the start or end, there may be any amount of lines full of numbers.
//!
//! In the middle, there's a single line, which is expected to have the following setup:
//! ```text
//! layer1[layer2[layer3[anyword]]]
//! ```
//!
//! Any amount of whitespaces or newlines is allowed before or after each `layerX` word.
//! `layer3` may contain any word that doesn't contain a `ü`.
//!
//! For example this is a valid input:
//! ```
//! 10
//! 20
//! layer1[   layer2
//! [layer3[somelongword bla bla bla ]]
//! ]
//! 30
//! 40
//! ```
use std::fs::read_to_string;
#[cfg(test)]
use std::path::PathBuf;

use alpm_parsers::error::Input;
use insta::assert_snapshot;
use parser::document;
use rstest::rstest;
use testresult::TestResult;

#[rstest]
fn ensure_parse_errors(#[files("tests/error_input/*")] case: PathBuf) -> TestResult {
    // Disable colored output, so that no ANSI escape codes end up in the snapshots.
    colored::control::set_override(false);

    // Read the input file and parse it.
    let input = read_to_string(&case)?;
    let result = document(&mut Input::new(&input));

    // Make sure there're no parse errors
    let Err(error) = result else {
        panic!("The parser succeeded even though it should've failed parsing.");
    };

    // Unwrap winnow's `ErrMode`, as we're only interested in our own error type.
    let Ok(error) = error.into_inner() else {
        panic!("Got unexpected incomplete parse error.");
    };

    let name = case
        .file_stem()
        .expect("path to have a file stem")
        .to_str()
        .expect("path to contain only Unicode characters");

    // Run the tests with the input being displayed as the description.
    // This makes reviewing this whole stuff a lot easier.
    // Also remove the usual module prefix by explicitly setting the snapshot path.
    // This isn't necessary, as we're already manually sorting snapshots by test scenario.
    let input_clone = input.clone();
    insta::with_settings!({
        description => input_clone,
        snapshot_path => "error_snapshots",
        prepend_module_to_snapshot => false,
    }, {
        assert_snapshot!(name, format!("{error}"));
    });

    Ok(())
}

/// The parsers for the imaginary document format
pub mod parser {
    use alpm_parsers::prelude::*;
    use winnow::{
        ascii::{digit1, multispace0, newline},
        combinator::{eof, repeat},
        error::ErrMode,
        stream::AsChar,
        token::take_while,
    };

    /// Parses the imaginary document
    pub fn document<'a>(input: &mut Input<'a>) -> PResult<'a, ()> {
        let parser = |input: &mut Input<'a>| -> PResult<'a, ()> {
            number_lines.parse_next(input)?;

            blocks.parse_next(input)?;

            (multispace0, number_lines).parse_next(input)?;

            eof.label("end of document")
                .expected_text("a line full of numbers")
                .expected_text("the end of the document")
                .parse_next(input)?;

            Ok(())
        };

        parser.layer("document").parse_next(input)
    }

    /// A simple parser that parses any amount of lines full of numbers.
    fn number_lines<'a>(input: &mut Input<'a>) -> PResult<'a, ()> {
        repeat(0.., (digit1, newline)).parse_next(input)
    }

    /// Creates a parser for a single `layerX[…]` block.
    fn block<'a, O>(
        name: &'static str,
        description: &'static str,
        inner: impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>>,
    ) -> impl Parser<Input<'a>, O, ErrMode<ParseStack<'a>>> {
        let mut inner = inner;

        let parser = move |input: &mut Input<'a>| -> PResult<'a, O> {
            // The keyword that names this block, e.g. `layer1`.
            (
                name.label(format!("{name} keyword")).expected_string(name),
                multispace0,
            )
                .parse_next(input)?;

            // The opening bracket, which starts this block's content.
            '['.label(format!("start of the {name} block"))
                .expected_char('[')
                .parse_next(input)?;

            // The block's content, which may be surrounded by whitespace.
            let content = (multispace0, inner.by_ref(), multispace0)
                .map(|(_, content, _)| content)
                .parse_next(input)?;

            // The closing bracket, which ends this block's content.
            ']'.label(format!("end of the {name} block"))
                .expected_char(']')
                .parse_next(input)?;

            Ok(content)
        };

        (multispace0, parser.description(description).layer(name)).map(|(_, content)| content)
    }

    /// Parses the nested block structure in the middle of a document.
    fn blocks<'a>(input: &mut Input<'a>) -> PResult<'a, &'a str> {
        block(
            "layer1",
            "The layer1 block, which wraps a single layer2 block",
            block(
                "layer2",
                "The layer2 block, which wraps a single layer3 block",
                block(
                    "layer3",
                    "The layer3 block, which wraps the payload word",
                    payload,
                ),
            ),
        )
        .parse_next(input)
    }

    /// Parses the payload word inside the innermost `layer3` block.
    ///
    /// Accepts any input but `]`, `ü` and newlines.
    fn payload<'a>(input: &mut Input<'a>) -> PResult<'a, &'a str> {
        take_while(1.., |char: char| {
            char != ']' && char != 'ü' && !char.is_newline()
        })
        .label("payload word")
        .expected_text("a word that contains neither brackets nor whitespace")
        .layer("payload")
        .parse_next(input)
    }
}
