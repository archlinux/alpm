/// Implement `From` for `$outer -> $inner` and `$inner -> $outer`
///
/// `$outer` is expected to be a newtype struct around $inner.
///
/// Example:
/// ```
/// import crate::impl_from;
///
/// struct MyType;
/// struct MyNewType(MyType);
///
/// impl_from(MyNewtype, MyType);
/// ```
macro_rules! impl_from {
    ($outer:ident, $inner:ty) => {
        impl From<$inner> for $outer {
            fn from(inner: $inner) -> Self {
                $outer(inner)
            }
        }

        impl From<$outer> for $inner {
            fn from(outer: $outer) -> Self {
                outer.0
            }
        }
    };
}

pub(crate) use impl_from;
