use std::collections::HashSet;

use serde::{Serialize, Serializer};

/// Serializes a [`HashSet`] as [`Vec`].
///
/// Converts `value` to a `Vec<T>` and calls `Vec::sort_unstable` on it.
/// This is necessary to guarantee idempotent ordering in serialized output.
///
/// We use `sort_unstable`, as we know that all elements are unique.
///
/// # Errors
///
/// Returns an error if serializing the `Vec<T>` fails.
pub fn ordered_hashset<S, V: Serialize + Ord>(
    value: &HashSet<V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut list: Vec<&V> = value.iter().collect();
    list.sort_unstable();
    list.serialize(serializer)
}

/// Serializes an `Override<HashSet<V>>` as `Override<Vec<V>>`.
///
/// Converts `value` to an `Override<Vec<V>>` and calls `Vec::sort_unstable` on it.
/// This is necessary to guarantee idempotent ordering in the serialized output.
///
/// We use `sort_unstable`, as we know that all elements are unique.
///
/// # Errors
///
/// Returns an error if serializing the `Override<Vec<V>>` fails.
pub fn ordered_optional_hashset<S, V: Serialize + Ord>(
    value: &Option<HashSet<V>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Option::None => value.serialize(serializer),
        Option::Some(value) => {
            let mut list: Vec<&V> = value.iter().collect();
            list.sort_unstable();
            let option = Some(value);
            option.serialize(serializer)
        }
    }
}
