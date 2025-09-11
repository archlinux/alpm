/// Implement two-way From conversions between a newtype and its inner type.
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

/// Convert a Vec of one type into a Vec of another type using the From trait.
/// When called without arguments, it returns a closure that can be used
/// e.g. when mapping an Option.
/// Convenient for interacting with Python lists.
macro_rules! vec_convert {
    ($vec:expr) => {
        $vec.into_iter().map(From::from).collect::<Vec<_>>()
    };
    () => {
        |v| vec_convert!(v)
    };
}

/// Same as vec_convert, but for BTreeMap.
/// Convenient for interacting with Python dicts.
macro_rules! btree_convert {
    ($btree:expr) => {
        $btree
            .into_iter()
            .map(|(k, v)| (From::from(k), From::from(v)))
            .collect::<BTreeMap<_, _>>()
    };
    () => {
        |b| btree_convert!(b)
    };
}

pub(crate) use btree_convert;
pub(crate) use impl_from;
pub(crate) use vec_convert;
