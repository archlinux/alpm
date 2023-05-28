// SPDX-FileCopyrightText: 2023 David Runge <dvzrv@archlinux.org>
// SPDX-License-Identifier: LGPL-3.0-or-later

/// A convenient way to create a regular expression only once
///
/// A string literal as input is used to define the regular expression.
/// With the help of OnceCell the regular expression is created only once.
///
/// ## Examples
/// ```
/// #[macro_use] extern crate alpm_types;
///
/// let re = regex_once!("^(foo)$");
/// assert!(re.is_match("foo"));
/// ```
macro_rules! regex_once {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub(crate) use regex_once;