use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use mandown::convert;

/// Render a man page
fn render_manpage(input: &Path, output: &Path, name: &str, section: u8) {
    let contents = read_to_string(input)
        .unwrap_or_else(|_| panic!("Error occurred reading markdown file {:?}", input));
    let manpage = convert(contents.as_str(), name, section);
    let mut output_file = File::create(output)
        .unwrap_or_else(|_| panic!("Error occurred creating man page file {:?}", output));
    output_file
        .write_all(manpage.as_bytes())
        .unwrap_or_else(|_| panic!("Error occurred writing to man page file {:?}", output));
}

fn main() {
    render_manpage(
        Path::new("resources/specification/BUILDINFOv1.5.md"),
        Path::new("resources/specification/BUILDINFOv1.5"),
        "BUILDINFOv1",
        5,
    );
}
