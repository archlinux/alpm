/// Implements [`From`] for `$outer -> $inner` and `$inner -> $outer`
///
/// `$outer` is expected to be a newtype struct around `$inner`.
///
/// # Examples
///
/// ```
/// use crate::impl_from;
///
/// struct MyType;
/// struct MyNewType(MyType);
///
/// impl_from(MyNewType, MyType);
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

/// Convert a [`Vec<T>`] into a [`Vec<D>`] using the [`From<T>`] for `D`.
/// When called without arguments, it returns a closure that can be used
/// e.g. when mapping an [`Option`].
/// Convenient for interacting with Python `list`s.
///
/// # Examples
///
/// ```
/// use crate::vec_convert;
/// # use crate::impl_from;
/// # struct MyType;
/// # struct MyNewType(MyType);
/// # impl_from(MyNewtype, MyType);
///
/// // Convert a `Vec<MyType>` into a `Vec<MyNewType>`
/// let my_types: Vec<MyType> = vec![MyType, MyType];
/// let my_new_types: Vec<MyNewType> = vec_convert!(my_types);
///
/// // Convert an `Option<Vec<MyType>>` into an `Option<Vec<MyNewType>>`
/// let my_types_opt: Option<Vec<MyType>> = Some(my_types);
/// let my_new_types_opt: Option<Vec<MyNewType>> = my_types_opt.map(vec_convert!());
/// ```
macro_rules! vec_convert {
    ($vec:expr) => {
        $vec.into_iter().map(From::from).collect::<Vec<_>>()
    };
    () => {
        |v| vec_convert!(v)
    };
}

/// Convert a [`BTreeMap<A, B>`] into a [`BTreeMap<C, D>`] using [`From<A>`] for `C` and [`From<B>`]
/// for `D`. When called without arguments, it returns a closure that can be used
/// e.g. when mapping an [`Option`].
/// Convenient for interacting with Python `dict`s.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use crate::vec_convert;
/// # use crate::impl_from;
/// # struct TypeA;
/// # struct TypeC(TypeA);
/// # impl_from(TypeC, TypeA);
/// # struct TypeB;
/// # struct TypeD(TypeB);
/// # impl_from(TypeD, TypeB);
///
/// // Convert a `BTreeMap<TypeA, TypeB>` into a `BTreeMap<TypeC, TypeD>`
/// let mut my_map: BTreeMap<TypeA, TypeB> = BTreeMap::new();
/// my_map.insert(TypeA, TypeB);
/// let my_new_map: BTreeMap<TypeC, TypeD> = btree_convert!(my_map);
///
/// // Convert an `Option<BTreeMap<TypeA, TypeB>>` into an `Option<BTreeMap<TypeC, TypeD>>`
/// let my_map_opt: Option<BTreeMap<TypeA, TypeB>> = Some(my_map);
/// let my_new_map_opt: Option<BTreeMap<TypeC, TypeD>> = my_map_opt.map(btree_convert!());
/// ```
///
/// [`BTreeMap<A, B>`]: std::collections::BTreeMap
/// [`BTreeMap<C, D>`]: std::collections::BTreeMap
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
