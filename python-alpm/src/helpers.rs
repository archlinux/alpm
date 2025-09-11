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
