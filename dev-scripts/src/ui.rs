use indicatif::{ProgressBar, ProgressStyle};

/// Get a styled indicatif progress bar for reuse across the project.
pub fn get_progress_bar(items: u64) -> ProgressBar {
    let bar = ProgressBar::new(items);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .expect("This progress syntax is valid"),
    );

    bar
}
