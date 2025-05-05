# NAME

pkgver - upstream version information for ALPM based packages.

# DESCRIPTION

The **pkgver** format is a version format, that is used for representing upstream version information.
This format is used in build scripts or file formats for package data description or reproduction.

A **pkgver** value is represented by a string, consisting of ASCII characters, excluding the ':', '/', '-' or any whitespace characters.
The **pkgver** value must be at least one character long, and must not start with a '.' sign.
If an upstream version contains an invalid character, it is advised to replace it with a valid one or to remove it.

# EXAMPLES

```
"1.0.0"
```

```
"1.0.0alpha"
```

# PACKAGE VERSION COMPARISON

The algorithm used to compare two package versions is based on the **vercmp** tool, which is itself based of RPM's **rpmvercmp** algorithm.

The **ALPM-rs** project provides a different algorithmic approach that mirrors the behavior of **vercmp**, and the following sections explain how this algorithm works.

The algorithm compares two version strings **A** and **B**.
Briefly summarized and simplified, the algorithm splits each version string into segments and compares the segment pairs of the two versions with each other.

## SEGMENTS

Versions can be split into segments, which can be further split into sub-segments if they're alphanumeric.

Segments in version strings are delimited by a dot `.` character.
For example, the version `1.0.0alpha.` would result in the segments `"1"`, `"0"`, `"0alpha"` and `""`. Note that a trailing `.` results in an empty trailing segment.

Sub-segments are used to further split version segments that are alphanumeric.
For example, the string `0alpha` would result in the sub-segments `"0"` and `"alpha"`.

### (SUB-)SEGMENT SPLITTING ALGORITHM

Version strings are split according to the following rules:

- Any non-alphanumeric character acts as a delimiter (e.g. `.`, `-`, `$`, `🐶`, `🐱`, etc.).
  **WARNING:** The old **vercmp** tool is not UTF-8 aware. This results in a 4-byte UTF-8 character being interpreted as 4 delimiters, even though it's only a single UTF-8 character.
  The **ALPM-rs** version comparison respects UTF-8 characters and handles them correctly as a single delimiter.
- There's no differentiation between different delimiters. (e.g. `"$$$"` == `"..."` == `".$-"`)
- Each segment also contains information about the number of leading delimiters in that segment, e.g. `---` would be three leading delimiters.
  Leading delimiters that directly follow after one another are grouped together instead of creating empty segments in between.
  The amount of these delimiters is important, as it plays a role in the algorithm that determines which version is newer.
  For example, `"1...a"` could be represented as:

  ```json
  [
    {
      "text": "1",
      "delimiters": 0
    },
    {
      "text": "a",
      "delimiters": 3
    }
  ]
  ```

- Alphanumeric strings are split into individual sub-segments. This is done by walking over each segment and splitting it every time a switch from alphabetic to numeric is detected or vice versa.
  For example, "`1.1foo123..20`" could be represented as:

  ```json
  [
    {
        "text": "1",
        "delimiters": 0
    },
    {
        "sub_segments": ["1", "foo", "123"]
        "delimiters": 1
    ],
    {
        "text": "20",
        "delimiters": 2
    },
  ]
  ```

- Trailing delimiters are encoded as a segment with an empty string.
  For example, "`1.`" could be represented as:

  ```json
  [
    {
      "text": "1",
      "delimiters": 0
    },
    {
      "text": "",
      "delimiters": 1
    }
  ]
  ```

## COMPARISON BEHAVIOR

When comparing two versions, both versions are split into their segments and sub-segments.
The (sub-)segments of both versions are then compared one after another until one of the (sub-)segments is smaller/bigger than the other or until one or both of the version strings end.

As a general rule of thumb, a version is considered "newer" _as soon as_ a (sub-)segment is found that is "greater" than that of its counterpart. The comparison algorithm then short-circuits.

General comparison rules are:

- When two numbers are compared, the bigger number is newer (`2 > 1`).
- When numbers are compared, leading zeros are ignored (`0001 == 1`).
- When a number and an alphabetic string are compared, the number is always considered newer (`1 > zeta`).
- When alphabetic strings are compared, simple string ordering is performed (`b > a` and `aab > aaa`).

### SIMPLE SEGMENT COMPARISON

The simplest comparison is when both segments that are compared don't have any subsections.
Examples for this are:

```
1.0.0 < 1.1.0
```

For `1.0.0` and `1.1.0`, the first segments `1` and `1` are equal.
The second segments `0` and `1` are compared, resulting in `1 > 0`.
The comparison concludes with `1.0.0 < 1.1.0`.
The last segment is not considered.

```
1.2.0 > 1.foo.0
```

For `1.2.0` and `1.foo.0`, the first segments `1` and `1` are equal.
The second segments `2` and `foo` are compared, resulting in `2 > foo`.
The comparison concludes with `1.2.0 > 1.foo.0`.
The last segment is not considered.

```
foo.0 > boo.0
```

For `boo.0` and `foo.0`, the first segments `boo` and `foo` are compared, resulting in `foo > boo`.
The comparison concludes with `foo.0 > boo.0`.
The last segment is not considered.

```
1.0 == 1.0
```

They're equal.

### (SUB-)SEGMENT COMPARISON

Sub-segment comparison follows very similar rules to simple segment comparison, but has an edge-case when one version has more sub-segments than another, which leads to sub-segments and segments being compared.

Examples:

```
alpha0 < beta0
```

The algorithm compares the sub-segments `alpha` and `beta`, resulting in `beta > alpha`
The comparison concludes with `alpha0 < beta0`.
The last sub-segment is not considered.

```
alpha1 < alpha02
```

The first sub-segments `alpha` and `alpha` are equal.
The second sub-segments `1` and `02` are compared and result in `1 < 2`.
The comparison concludes with `alpha1 < alpha02`.

```
1alpha0 < 2alpha0
```

The first sub-segments `1` and `2` are compared, resulting in `1 < 2`.
The comparison concludes with `1alpha0 < 2alpha0`.
The other sub-segments are not considered.

```
1.alpha0 < 1.alpha.0
```

If one of the versions has more _sub-segments_ than the other and the other has a _segment_ instead. The segment is considered _bigger_ than the sub-segment.
The first sub-segments `alpha` and `alpha` are equal.
Next, the sub-segment `5` of `alpha5` and the second segment `1` of `alpha.1` are compared. Since one is a segment and the other a sub-segment, their actual contents are ignored, and the decision is based solely on the rule that a segment takes precedence over a sub-segment.
As a result, the comparison concludes with `alpha5.1 < alpha.1`.
The second segment of `alpha5.1` is not considered.

### SPECIAL CASES

#### Multiple delimiters between segments

If a version has multiple delimiters between segments and more than its counterpart at the same position, it is always considered newer. The reason for this behavior is unclear and it's uncertain whether it was intentional.

Examples:

```
1...0 > 1.2
1.0..0 > 1.0.15
```

For `1...0` and `1.2`, the first segments `1` and `1` are equal.
Both versions have another segment afterwards, but `1...0` has three delimiters while `1.2` only has one and `three delimiters > one delimiter`.
The comparison concludes with `1...0 > 1.2`.

#### One version has more segments

Examples:

```
1 < 1.0
1 < 1.foo
```

The first segments `1` and `1` are equal.
The first version than hits the end while the second has another segment.
Another segment is always bigger than no segment, which also applies for foobetic segments.
The comparison concludes with `1 < 1.0` and `1 < 1.foo`.

```
1.0 > 1.0foo.2
```

The first segments `1` and `1` are equal.
The next sub-segments `0` and `0` are also equal.
The first version now hits the end, while the second version has another sub-segment `foo`.
Since that sub-segment is alphabetical, it's considered _older_. This special case was historically introduced to catch alpha/beta releases that were often marked via a `alpha` suffix _without_ delimiter such as `1.0.0alpha.1`, which sadly holds no longer true in modern **semantic versioning**.
As a result, the comparison concludes with `1.0 > 1.0foo.2`.
Note that the last segment of `1.0foo.2` is not considered.

```
1.foo < 1.foo2
```

The first segments `1` and `1` are equal.
The next sub-segments `foo` and `foo` are also equal.
The first version now hits the end, while the second version has another sub-segment `1`.
Since that sub-segment is numerical, it's considered _newer_.
As a result, the comparison concludes with `1.foo < 1.foo2`.

#### One or both versions have trailing delimiters

If one or both of the versions have trailing delimiter, the behavior is very similar to the section above, but there're some minor differences.

Examples:

```
1... == 1.
```

The first segments `1` and `1` are equal.
Both versions now hit the end with an arbitrary amount of trailing delimiters.
Counterintuitively, the amount of trailing delimiters **does not** matter in this case, they're always considered equal.
The comparison concludes with `1... == 1.`.

```
1. > 1.foo.2
```

The first segments `1` and `1` are equal.
The first version now hits the end, while the second version has another segment `foo`.
This is similar to the case `1.0 > 1.0foo.2` in the section above, except that the same rule now applies **despite** `foo` being a segment instead of a sub-segment. Since `foo` is alphabetical, it's considered older.
The comparison concludes with `1. > 1.foo.2`.

```
1. < 1.2
1. < 1.2foo
```

The first segments `1` and `1` are equal.
The first version now hits the end, while the second version has a (sub-)segment `2`.
As `2` is numerical, it's considered newer.
The comparison concludes with `1. < 1.2foo`.

```
1.alpha. < 1.alpha0
```

The first segments `1` and `1` are equal.
The next sub-segments `alpha` and `alpha` are also equal.
The first version now hits the end, while the second version has a (sub-)segment `0`.
As `0` is numerical, it's considered newer.
The comparison concludes with `1.alpha. < 1.alpha0`.

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-epoch**(7), **alpm-package-version**(7), **alpm-pkgrel**(7), **vercmp**(8)

# NOTES

1. **rpmvercmp**

   https://fedoraproject.org/wiki/Archive:Tools/RPM/VersionComparison

2. **semantic versioning**

   https://semver.org/
