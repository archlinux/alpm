use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};

use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    combinator::{alt, eof, preceded, repeat, seq},
    token::{one_of, rest},
};

/// Kind of change that was made to a path.
///
/// See the [`--itemize-changes` section in `man 1 rsync`](https://man.archlinux.org/man/rsync.1#itemize-changes).
/// This type corresponds to the `Y` placeholder in the format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ChangeKind {
    /// A file is being transferred to the remote host.
    Sent,
    /// A file is being transferred to the local host.
    Received,
    /// A local change is occurring for the item.
    LocalChange,
    /// Item is a hard link to another item.
    Hardlink,
    /// The item is not being updated, but its attributes may be modified.
    NotUpdated,
}

/// Kind of path that a change was made to.
///
/// The `rsync` invocation in this crate should only emit [`PathKind::File`].
/// Any other values are unexpected and considered errors.
///
/// See the [`--itemize-changes` section in `man 1 rsync`](https://man.archlinux.org/man/rsync.1#itemize-changes).
/// This type corresponds to the `X` placeholder in the format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathKind {
    /// A file is changed.
    File,
    /// A directory is changed.
    Directory,
    /// A symlink is changed.
    Symlink,
    /// A device is changed.
    Device,
    /// A special file is changed.
    ///
    /// This can include named sockets and FIFOs.
    SpecialFile,
}

/// Change information reported by `rsync`.
///
/// See the [`--itemize-changes` section in `man 1 rsync`](https://man.archlinux.org/man/rsync.1#itemize-changes).
#[derive(Clone, Debug)]
pub(crate) enum Report<'a> {
    /// `rsync` message string.
    Message(&'a OsStr),
    /// Changes have been made to a local path.
    PathChange {
        /// Kind of change that is being made.
        change_kind: ChangeKind,
        /// Kind of path that is being changed.
        path_kind: PathKind,
        /// [`Path`] to the item in question.
        path: &'a Path,
    },
    /// `rsync` did not report any changes.
    Empty,
}

impl<'a> Report<'a> {
    /// Parse a [`Self`] from one line of `rsync --itemize-changes`.
    ///
    /// Callers should be careful to remove trailing newlines from `rsync` output
    /// as this function will *not* remove them.
    ///
    /// For accepted inputs, see the
    /// [`--itemize-changes` section in `man 1 rsync`](https://man.archlinux.org/man/rsync.1#itemize-changes).
    ///
    /// # Errors
    ///
    /// Throws an error if `input` is of an unknown format
    pub(crate) fn parser(mut line: &'a [u8]) -> winnow::ModalResult<Self> {
        let line = &mut line;
        let mut change_kind = alt((
            b'>'.value(ChangeKind::Received),
            b'c'.value(ChangeKind::LocalChange),
            b'h'.value(ChangeKind::Hardlink),
            b'<'.value(ChangeKind::Sent),
            b'.'.value(ChangeKind::NotUpdated),
        ));

        let mut path_kind = alt((
            b'f'.value(PathKind::File),
            b'd'.value(PathKind::Directory),
            b'L'.value(PathKind::Symlink),
            b'D'.value(PathKind::Device),
            b'S'.value(PathKind::SpecialFile),
        ));

        let mut ignored = (
            // attribute changes
            repeat::<_, _, (), _, _>(
                9,
                one_of((b'a'..=b'z', b'A'..=b'Z', b'.', b'+', b' ', b'?')),
            ),
            // separator space
            b" ",
        );

        alt((
            // empty string
            eof.value(Self::Empty),
            // rsync message
            preceded(b'*', rest.map(OsStr::from_bytes).map(Self::Message)),
            // item change
            seq!(Self::PathChange {
                change_kind: change_kind,
                path_kind: path_kind,
                _: ignored,
                path: rest.map(|p: &[u8]| Path::new(OsStr::from_bytes(p))),
            }),
        ))
        .parse_next(line)
    }

    /// Check if file content was changed in such a way that it should be extracted again.
    ///
    /// Returns `Ok(Some(path))`, where `path` points to the file if there were changes.
    /// Otherwise `Ok(None)`.
    ///
    /// This does **not** consider the following to be "relevant" changes:
    /// - Directory creation or other ["local changes" as defined by rsync](https://man.archlinux.org/man/rsync.1#o~61).
    /// - Sending to remote. This should *not* be possible when using this tool.
    /// - Attribute changes.
    /// - File deletion.
    ///
    /// # Errors
    ///
    /// Returns an error if `self`:
    /// - Holds a non-deletion message from rsync.
    /// - Reports a change to a non-file object. See [`PathKind`].
    pub(crate) fn file_content_updated(&self) -> Result<Option<&Path>> {
        use Report::*;

        match self {
            // a file was changed
            PathChange {
                change_kind: ChangeKind::Received | ChangeKind::Hardlink,
                path_kind: PathKind::File,
                path,
            } => Ok(Some(path)),
            // a non-file path was changed
            PathChange {
                change_kind: ChangeKind::Received | ChangeKind::Hardlink,
                path_kind,
                path,
            } => Err(anyhow!(
                "Got unexpected path kind {path_kind:?} in rsync change list for '{path:?}'"
            )),
            // something was sent to the remote
            PathChange {
                change_kind: ChangeKind::Sent,
                path,
                ..
            } => {
                log::warn!(
                    "Path '{path:?}' reported as changed on the remote host, this should not happen",
                );
                Ok(None)
            }
            // a message other than deletion was returned
            Message(msg) if !msg.as_bytes().starts_with(b"deleting") => Err(anyhow!(
                "rsync message found while looking for changes: '{}'",
                msg.to_string_lossy()
            )),
            // all other cases are considered "unchanged"
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::proptest;
    use rstest::rstest;
    use testresult::TestResult;

    use super::*;

    proptest! {
        #[test]
        fn rsync_file_no_panic(input: Vec<u8>) {
            // print to guarantee that the compiler doesn't optimise the call away
            println!("{:?}", Report::parser(&input));
        }

        #[test]
        fn rsync_parses(magic_str in "[<>ch.][fdLDS][.+ ?a-z]{9} ", path: PathBuf) {
            let mut input = magic_str.into_bytes();
            input.extend_from_slice(path.as_os_str().as_bytes());
            // unwrap to panic if the call produces an error
            Report::parser(&input).unwrap();
        }
    }

    #[rstest]
    #[case::changed(b">f+++++++++ file.name", b"file.name")]
    #[case::hardlink(b"hf+++++++++ hard.link", b"hard.link")]
    #[case::file_with_space(b">f+++++++++ file with space.ext", b"file with space.ext")]
    #[case::file_with_trailing_newline(
        b">f+++++++++ file_with_newline.ext\n",
        b"file_with_newline.ext\n"
    )]
    #[case::blank_filename(b">f+++++++++ ", b"")]
    fn rsync_changed(#[case] input: &[u8], #[case] changed: &[u8]) -> TestResult {
        assert_eq!(
            Report::parser(input)?
                .file_content_updated()?
                .ok_or("rsync reported no change")?
                .as_os_str()
                .as_bytes(),
            changed
        );
        Ok(())
    }

    #[rstest]
    #[case::received(
        b">f+++++++++ received_file.txt",
        ChangeKind::Received,
        PathKind::File,
        b"received_file.txt"
    )]
    #[case::sent(
        b"<f+++++++++ sent_file.txt",
        ChangeKind::Sent,
        PathKind::File,
        b"sent_file.txt"
    )]
    #[case::local(
        b"cL+++++++++ changed.md",
        ChangeKind::LocalChange,
        PathKind::Symlink,
        b"changed.md"
    )]
    #[case::hardlink(
        b"hf+++++++++ hard.link",
        ChangeKind::Hardlink,
        PathKind::File,
        b"hard.link"
    )]
    #[case::not_updated(
        b".D+++++++++ /dev/sda0",
        ChangeKind::NotUpdated,
        PathKind::Device,
        b"/dev/sda0"
    )]
    #[case::directory(
        b"cd+++++++++ directory_one",
        ChangeKind::LocalChange,
        PathKind::Directory,
        b"directory_one"
    )]
    fn rsync_change_kind(
        #[case] input: &[u8],
        #[case] expected_change_kind: ChangeKind,
        #[case] expected_path_kind: PathKind,
        #[case] changed_name: &[u8],
    ) -> TestResult {
        let Report::PathChange {
            change_kind,
            path_kind,
            path,
        } = Report::parser(input)?
        else {
            panic!("rsync didn't report a change");
        };
        assert_eq!(change_kind, expected_change_kind);
        assert_eq!(path_kind, expected_path_kind);
        assert_eq!(path.as_os_str().as_bytes(), changed_name);
        Ok(())
    }

    #[rstest]
    #[case::attr_change(b".f+++++++++ file.name")]
    #[case::changed_remotely(b"<f+++++++++ file.name")]
    #[case::empty(b"")]
    #[case::deleted(b"*deleting file.name")]
    fn rsync_unchanged(#[case] input: &[u8]) -> TestResult {
        assert_eq!(Report::parser(input)?.file_content_updated()?, None);
        Ok(())
    }

    #[rstest]
    #[case::no_space(b">f+++++++++file.name")]
    #[case::invalid_first_char(b"Bf+++++++++ file.name")]
    #[case::no_file(b">f+++++++++")]
    fn rsync_report_parse_error(#[case] input: &[u8]) {
        assert!(Report::parser(input).is_err());
    }
}
