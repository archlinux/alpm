use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    string::FromUtf8Error,
};

use thiserror::Error;

use crate::create::types::ArchVec;

#[derive(Error, Debug, Clone)]
pub enum LintKind {
    #[error("unknown fragment: {0:#}")]
    UnknownFragment(String),
    #[error("wrong value type: {0:#} {1} {2}")]
    WrongValueType(String, String, String),
    #[error("can't be architecture specific: {0:#} {1}")]
    CantBeArchitectureSpecific(String, String),
    #[error("can't be architecture specific")]
    CantBeArchitectureSpecificAny,
    #[error("{0:#}")]
    VariableCantBeInPackageFunction(String),
    #[error("{0:#}")]
    VariabeContainsNewlines(String),
    #[error("{0:#}")]
    VariabeContainsEmptyString(String),
    #[error("conflicting package functions")]
    ConflictingPackageFunctions,
    #[error("wrong package function format")]
    WrongPackgeFunctionFormat,
    #[error("{0:#}")]
    MissingPackageFunction(String),
    #[error("{0:#}")]
    MissingFile(String, String),
    #[error("any architecture can't be combined with others")]
    AnyArchWithOthers,
    #[error("{0:#}")]
    BackupHasLeadingSlash(String),
    #[error("{0:#}")]
    IntegrityChecksMissing(String),
    #[error("{0:#} {1}")]
    StartsWithInvalid(String, String),
    #[error("{0:#} {1}")]
    InvalidChars(String, String),
    #[error("{0:#}")]
    InvalidPkgver(String),
    #[error("{0:#}")]
    InvalidPkgrel(String),
    #[error("{0:#} {1}")]
    AsciiOnly(String, String),
    #[error("{0:#} {1}")]
    IntegrityChecksDifferentSize(String, String),
    #[error("{0:#}")]
    InvalidPkgExt(String),
    #[error("{0:#}")]
    InvalidSrcExt(String),
    #[error("{0:#}")]
    InvalidEpoch(String),
    #[error("{0:#}")]
    InvalidIntegrityCheck(String),
}

pub(crate) type LintResult<T> = std::result::Result<T, LintKind>;

pub(crate) static PKGBUILD_SCRIPT: &str = include_str!("pkgbuild-bridge.sh");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Array(Vec<String>),
    Map(HashMap<String, String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub arch: Option<String>,
    pub value: Value,
}

impl Variable {
    pub fn name_arch(&self) -> String {
        if let Some(arch) = &self.arch {
            format!("{}_{}", self.name, arch)
        } else {
            self.name.clone()
        }
    }

    pub fn assert_no_arch(&self) -> LintResult<()> {
        if self.arch.is_some() {
            return Err(LintKind::CantBeArchitectureSpecific(
                self.name.to_string(),
                self.name_arch(),
            ));
        }

        Ok(())
    }

    pub fn get_arch_array(self) -> LintResult<ArchVec<String>> {
        match self.value {
            Value::Array(a) => Ok(ArchVec::from_vec(self.arch, a)),
            _ => Err(LintKind::WrongValueType(
                self.name_arch(),
                "array".to_string(),
                self.kind().to_string(),
            )),
        }
    }

    pub fn get_array(self) -> LintResult<Vec<String>> {
        self.assert_no_arch()?;
        match self.value {
            Value::Array(a) => Ok(a),
            _ => Err(LintKind::WrongValueType(
                self.name_arch(),
                "array".to_string(),
                self.kind().to_string(),
            )),
        }
    }

    pub fn get_path_array(self) -> LintResult<Vec<PathBuf>> {
        self.get_array()
            .map(|v| v.into_iter().map(PathBuf::from).collect())
    }

    pub fn get_string(self) -> LintResult<String> {
        self.assert_no_arch()?;
        match self.value {
            Value::String(s) => Ok(s),
            _ => Err(LintKind::WrongValueType(
                self.name_arch(),
                "string".to_string(),
                self.kind().to_string(),
            )),
        }
    }

    fn kind(&self) -> &'static str {
        match self.value {
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Map(_) => "map",
        }
    }
}

#[derive(Default, Debug)]
pub struct FunctionVariables {
    pub function_name: String,
    pub variables: Vec<Variable>,
}

#[derive(Default, Debug)]
pub struct RawPkgbuild {
    pub variables: Vec<Variable>,
    pub function_variables: Vec<FunctionVariables>,
    pub functions: Vec<String>,
}

#[derive(Error, Debug)]
pub enum RawPkgbuildError {
    #[error("raw io error: {0:#}")]
    IOError(#[from] io::Error),
    #[error("parse error: {0:#}")]
    ParseError(#[from] ParseError),
    #[error("utf8 error: {0:#}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
}

impl RawPkgbuild {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, RawPkgbuildError> {
        let path = path.as_ref().canonicalize()?;
        let parent = path
            .parent()
            .ok_or_else(|| RawPkgbuildError::InvalidPath(path.to_path_buf()))?;

        let output = bash_output(Some(parent), &[&path], "dump")?;
        let pkgbuild: RawPkgbuild = RawPkgbuild::parse_processed_output(&output)?;
        Ok(pkgbuild)
    }

    fn parse_processed_output(s: &str) -> Result<Self, ParseError> {
        let mut data = Self::default();

        for line in s.lines() {
            parse_line(&mut data, line)?;
        }

        Ok(data)
    }
}

fn bash_output<P: AsRef<Path>>(
    dir: Option<&Path>,
    files: &[P],
    cmd: &str,
) -> Result<String, RawPkgbuildError> {
    let mut command = Command::new("bash");
    command
        .arg("--noprofile")
        .arg("--norc")
        .arg("-s")
        .arg("-")
        .arg(cmd);
    for file in files {
        command.arg(file.as_ref());
    }
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(PKGBUILD_SCRIPT.as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;
    let output = String::from_utf8(output.stdout)?;

    Ok(output)
}

fn words(line: &str) -> Result<Vec<String>, ParseError> {
    let mut words = Vec::new();

    let mut line = line.trim();

    while !line.is_empty() {
        if line.starts_with('"') {
            let mut word = String::new();
            let mut chars = line.chars();
            chars.next();

            loop {
                match chars.next() {
                    Some('\\') => match chars.next() {
                        Some('\\') => word.push('\\'),
                        Some('"') => word.push('"'),
                        Some('n') => word.push('\n'),
                        Some(c) => {
                            return Err(ParseError::new(
                                line,
                                ParseErrorKind::UnknownEscapeSequence(c),
                            ));
                        }
                        None => todo!(),
                    },
                    Some('"') => break,
                    Some(c) => word.push(c),
                    None => {
                        return Err(ParseError::new(
                            line,
                            ParseErrorKind::UnterminatedString(word.to_string()),
                        ));
                    }
                }
            }

            if !matches!(chars.next(), None | Some(' ')) {
                return Err(ParseError::new(
                    line,
                    ParseErrorKind::UnescapedQuoteInString(word.to_string()),
                ));
            }

            words.push(word.to_string());
            line = chars.as_str().trim_start()
        } else {
            let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
            words.push(word.to_string());
            line = rest.trim_start();
        }
    }

    Ok(words)
}

fn unexpected_word(line: &str, word: &str) -> ParseError {
    ParseError::new(line, ParseErrorKind::UnexpectedWord(word.to_string()))
}

fn end_of_words<I: Iterator<Item = String>>(line: &str, words: &mut I) -> Result<(), ParseError> {
    match words.next() {
        Some(w) => Err(unexpected_word(line, &w)),
        None => Ok(()),
    }
}

fn next_word<I: Iterator<Item = String>>(line: &str, words: &mut I) -> Result<String, ParseError> {
    match words.next() {
        Some(word) => Ok(word),
        None => Err(ParseError::new(line, ParseErrorKind::UnexpectedEndOfInput)),
    }
}

fn parse_line(data: &mut RawPkgbuild, line: &str) -> Result<(), ParseError> {
    let mut words = words(line)?.into_iter();

    match next_word(line, &mut words)?.as_str() {
        "VAR" => {
            let mut conf = false;

            let function = match next_word(line, &mut words)?.as_str() {
                "GLOBAL" => None,
                "CONFIG" => {
                    conf = true;
                    None
                }
                "FUNCTION" => Some(next_word(line, &mut words)?),
                w => return Err(unexpected_word(line, w)),
            };

            let kind = next_word(line, &mut words)?;
            let name = next_word(line, &mut words)?;

            let (name, arch) = if conf {
                (name, None)
            } else {
                match name.split_once('_') {
                    Some((name, arch)) => (name.to_owned(), Some(arch.to_string())),
                    None => (name, None),
                }
            };

            let value = match kind.as_str() {
                "STRING" => {
                    let value = Value::String(next_word(line, &mut words)?);
                    end_of_words(line, &mut words)?;
                    value
                }
                "ARRAY" => Value::Array(words.collect()),
                "MAP" => {
                    let mut map = HashMap::new();
                    while let Some(key) = words.next() {
                        let value = next_word(line, &mut words)?;
                        map.insert(key, value);
                    }
                    Value::Map(map)
                }
                w => return Err(unexpected_word(line, w)),
            };

            let variable = Variable { name, arch, value };

            if let Some(function) = function {
                match data
                    .function_variables
                    .iter_mut()
                    .find(|f| f.function_name == function)
                {
                    Some(f) => f.variables.push(variable),
                    None => data.function_variables.push(FunctionVariables {
                        function_name: function,
                        variables: vec![variable],
                    }),
                }
            } else {
                data.variables.push(variable);
            }
        }
        "FUNCTION" => {
            let function = parse_function(line, &mut words)?;
            data.functions.push(function);
        }
        w => return Err(unexpected_word(line, w)),
    }

    Ok(())
}

fn parse_function<I: Iterator<Item = String>>(
    line: &str,
    words: &mut I,
) -> Result<String, ParseError> {
    let word = next_word(line, words)?;
    end_of_words(line, words)?;
    Ok(word)
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: String,
    pub kind: ParseErrorKind,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to parse: {}", self.kind)
    }
}

impl ParseError {
    pub(crate) fn new<S: Into<String>>(line: S, kind: ParseErrorKind) -> Self {
        Self {
            line: line.into(),
            kind,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    #[error("unknown escape sequence: {0}")]
    UnknownEscapeSequence(char),
    #[error("unterminated string: {0}")]
    UnterminatedString(String),
    #[error("unescaped quote in string: {0}")]
    UnescapedQuoteInString(String),
    #[error("unexpected word: {0}")]
    UnexpectedWord(String),
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
}
