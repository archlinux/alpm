//! Helpers for the [knus] based deserialization of [KDL] documents.
//!
//! [KDL]: https://kdl.dev/
//! [knus]: https://docs.rs/knus/latest/knus/

use std::str::FromStr;

use knus::{
    ast::{Literal, TypeName},
    decode::{Context, Kind},
    errors::{DecodeError, ExpectedType},
    span::Spanned,
};

/// Emits an error if a [KDL type annotation] is attached to a scalar value.
///
/// We don't expect our types to have any type annotations.
/// So if we find one, we mark it as an error.
///
/// [KDL type annotation]: https://kdl.dev/spec/#section-3.8
pub(crate) fn deny_type_name(
    type_name: &Option<Spanned<TypeName>>,
    ctx: &mut Context,
    rust_type: &'static str,
) {
    if let Some(type_name) = type_name {
        ctx.emit_error(DecodeError::TypeName {
            span: *type_name.span(),
            found: Some((**type_name).clone()),
            expected: ExpectedType::no_type(),
            rust_type,
        });
    }
}

/// Decodes a scalar KDL value into `T` using the [`FromStr`] implementation of `T`.
///
/// # Errors
///
/// Returns an error if
/// - `value` is not a KDL string.
/// - `value` cannot be parsed into `T`
pub(crate) fn decode_str_scalar<T>(value: &Spanned<Literal>) -> Result<T, DecodeError>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match &**value {
        Literal::String(string) => string
            .parse()
            .map_err(|error| DecodeError::conversion(value, error)),
        _ => Err(DecodeError::scalar_kind(Kind::String, value)),
    }
}

/// Implements [`knus::DecodeScalar`] for a type using its [`FromStr`] implementation.
///
/// The decoder expects input types to be KDL strings and to not carry a KDL type annotation.
///
/// # Example
///
/// Implement DecodeScalar for `Name`:
///
/// ```norun
/// crate::util::knus::impl_decode_str_scalar!(Name);
/// ```
///
/// Generic types must specify their generics and bounds in square brackets in front of the type:
///
/// ```norun
/// crate::util::knus::impl_decode_str_scalar!([D: DigestString] Checksum<D>);
/// ```
macro_rules! impl_decode_str_scalar {
    (@impl [$($generics:tt)*] $type:ty) => {
        impl<$($generics)*> ::knus::DecodeScalar for $type {
            fn type_check(
                type_name: &Option<::knus::span::Spanned<::knus::ast::TypeName>>,
                ctx: &mut ::knus::decode::Context,
            ) {
                $crate::util::knus::deny_type_name(type_name, ctx, stringify!($type))
            }

            fn raw_decode(
                value: &::knus::span::Spanned<::knus::ast::Literal>,
                _ctx: &mut ::knus::decode::Context,
            ) -> Result<Self, ::knus::errors::DecodeError> {
                $crate::util::knus::decode_str_scalar(value)
            }
        }
    };
    ([$($bounds:tt)+] $type:ty) => {
        $crate::util::knus::impl_decode_str_scalar!(@impl [$($bounds)+] $type);
    };
    ($($type:ty),+ $(,)?) => {
        $($crate::util::knus::impl_decode_str_scalar!(@impl [] $type);)+
    };
}
pub(crate) use impl_decode_str_scalar;

/// Creates `T` using by parsing the value into usize and calling `From<usize>` on `T`.
///
/// # Errors
///
/// Returns an error if
/// - `value` is not a KDL Integer.
/// - `value` cannot be parsed into `usize`
pub(crate) fn decode_int_scalar<T>(value: &Spanned<Literal>) -> Result<T, DecodeError>
where
    T: From<usize>,
{
    match &**value {
        Literal::Int(integer) => {
            let value: usize = integer
                .try_into()
                .map_err(|error| DecodeError::conversion(value, error))?;

            Ok(T::from(value))
        }
        _ => Err(DecodeError::scalar_kind(Kind::String, value)),
    }
}

/// Implements [`knus::DecodeScalar`] for a type `T` where `T: From<usize>`.
///
/// The decoder expects input types to be KDL integer types and to not carry a KDL type annotation.
///
/// # Example
///
/// Implement DecodeScalar for `Name`:
///
/// ```norun
/// crate::util::knus::impl_decode_int_scalar!(Epoch);
/// ```
///
/// Generic types must specify their generics and bounds in square brackets in front of the type.
macro_rules! impl_decode_int_scalar{
    (@impl [$($generics:tt)*] $type:ty) => {
        impl<$($generics)*> ::knus::DecodeScalar for $type {
            fn type_check(
                type_name: &Option<::knus::span::Spanned<::knus::ast::TypeName>>,
                ctx: &mut ::knus::decode::Context,
            ) {
                $crate::util::knus::deny_type_name(type_name, ctx, stringify!($type))
            }

            fn raw_decode(
                value: &::knus::span::Spanned<::knus::ast::Literal>,
                _ctx: &mut ::knus::decode::Context,
            ) -> Result<Self, ::knus::errors::DecodeError> {
                $crate::util::knus::decode_int_scalar(value)
            }
        }
    };
    ([$($bounds:tt)+] $type:ty) => {
        $crate::util::knus::impl_decode_int_scalar!(@impl [$($bounds)+] $type);
    };
    ($($type:ty),+ $(,)?) => {
        $($crate::util::knus::impl_decode_int_scalar!(@impl [] $type);)+
    };
}

pub(crate) use impl_decode_int_scalar;
