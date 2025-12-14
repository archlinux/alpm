# NAME

pkgver - upstream version information for ALPM based packages.

# DESCRIPTION

The **pkgver** format is a version format, that is used for representing upstream version information.
This format is used in build scripts or file formats for package data description or reproduction.

A **pkgver** value is represented by a string, consisting of ASCII characters, excluding the ':', '/', '-', '<', '>', '=' or any whitespace characters.
The **pkgver** value must be at least one character long.
If an upstream version contains an invalid character, it is advised to replace it with a valid one or to remove it.

# EXAMPLES

```text
"1.0.0"
```

```text
"1.0.0alpha"
```

## Version comparison

The algorithm used for the comparison of two package versions is based on the **vercmp** tool, which itself is based on RPM's **rpmvercmp** algorithm in version 4.8.1.

The **ALPM** project provides a different algorithmic approach that mirrors the behavior of **vercmp**, and the following sections explain how this algorithm works.

The algorithm compares two version strings **A** and **B**.
In short, the algorithm splits each version string into segments and compares the segment pairs of the two versions with each other.

## Segments and sub-segments

Version strings can be split into segments, which can be further split into sub-segments if they are alphanumeric.

Segments in version strings are delimited by a dot `.` character.
For example, the version `1.0.0alpha.` would result in the segments `"1"`, `"0"`, `"0alpha"` and `""`. Note that a trailing `.` results in an empty trailing segment.

Sub-segments are used to further split version segments that contain **both** alphabetic and numeric characters.
For example, the string `0alpha` would result in the sub-segments `"0"` and `"alpha"`.

### Algorithm for splitting of version strings

Version strings are split according to the following rules:

- Any non-alphanumeric character (e.g. `.`, `-`, `$`, `🐱`, `🐶`, etc.) acts as a delimiter.
  The **ALPM** version comparison algorithm respects UTF-8 characters and handles them correctly as a single delimiter.
  **NOTE**: The **vercmp** tool is not UTF-8 aware, which results in a 4-byte UTF-8 character being interpreted as 4 delimiters, even though it is only a single UTF-8 character.
- Differing non-alphanumeric characters are not differentiated as delimiters (e.g. `"$$$"` == `"..."` == `".$-"`).
- Several subsequent delimiters are treated as a single delimiter when it comes to the splitting of version strings (e.g. both `1...a` and `1.a` are split into the two segments `1` and `a`).
- Each segment retains information about the number of delimiters that precede the segment.
  The number of delimiters is important, as it plays a role in the algorithm that determines which version is newer.
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
  
  This is done by walking over each character of such a segment and splitting it every time a switch from alphabetic to numeric (or vice versa) is detected.
  For example, the version string "`1.1foo123..20`" could be represented as:
  
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

- Trailing delimiters of a version string are represented by a segment with an empty string.
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

## Comparison behavior

Two version strings are compared by first splitting each into their **segments and sub-segments** based on the **algorithm for splitting of version strings**.
The segments and sub-segments from both sides are then compared sequentially with one another.
Comparison ends if a segment or sub-segment of one side is considered newer or older than that of the other, or if there are no further segments and sub-segments to compare.
Though there are a few rare exceptions to this general rule, which are listed in the **Comparison for special cases** section.

The following general comparison rules apply:

- Strings containing only numeric characters are compared as integers.
  Here, the larger number is considered newer (e.g. `2 > 1`).
- In strings containing only numeric characters, leading zeros are ignored (e.g. `0001 == 1`).
- When a string containing only numeric and one only containing alphabetic characters are compared, the former is always considered newer (e.g. `1 > zeta`).
- When strings containing only alphabetic characters are compared, simple string ordering is performed (e.g. `b > a` and `aab > aaa`).

### Comparison of version strings without sub-segments

The comparison between version strings without any sub-segments is simple.

Examples:

```text
1.0.0 < 1.1.0
```

For `1.0.0` and `1.1.0`, the first segments `1` and `1` are equal.
The second segments `0` and `1` are compared, resulting in `1 > 0`.
The comparison concludes with `1.0.0 < 1.1.0`.
The last segment is not considered.

```text
1.2.0 > 1.foo.0
```

For `1.2.0` and `1.foo.0`, the first segments `1` and `1` are equal.
The second segments `2` and `foo` are compared, resulting in `2 > foo`.
The comparison concludes with `1.2.0 > 1.foo.0`.
The last segment is not considered.

```text
foo.0 > boo.0
```

For `boo.0` and `foo.0`, the first segments `boo` and `foo` are compared, resulting in `foo > boo`.
The comparison concludes with `foo.0 > boo.0`.
The last segment is not considered.

```text
1.0 == 1.0
```

All compared segments are equal.

### Comparison of version strings with sub-segments

When version strings contain alphanumeric segments, these are split into sub-segments.
While the comparison behavior is similar to that of **comparison of version strings without sub-segments**, it features a special case where subsequent sub-segments are compared with segments.

Examples:

```text
alpha0 < beta0
```

The sub-segments `alpha` and `beta` are compared by applying simple string ordering, which results in `beta > alpha`
The comparison concludes with `alpha0 < beta0`.
The last sub-segment is not considered.

```text
alpha1 < alpha02
```

The first sub-segments `alpha` and `alpha` are equal.
The second sub-segments `1` and `02` are compared and result in `1 < 2`.
The comparison concludes with `alpha1 < alpha02`.

```text
1alpha0 < 2alpha0
```

The first sub-segments `1` and `2` are compared, resulting in `1 < 2`.
The comparison concludes with `1alpha0 < 2alpha0`.
The other sub-segments are not considered.

```text
alpha1 < alpha.0
```

The characters in the first sub-segment `alpha` and the first segment `alpha` are equal.
This leads to the special case of a **subsequent** sub-segment `1` and a **new** segment `0` being compared.
In this special case, the new segment is always considered newer and the contents of the sub-segment are ignored.
As a result, the comparison concludes with `alpha1 < alpha.0`.
The second sub-segment (`1`) of `alpha1` is not considered.

### Comparison for special cases

The version comparison algorithm offers a few (arguably surprising) special cases, that require further expansion.

#### Multiple delimiters between segments

When looking at the delimiters between segments in two version strings, the amount of delimiters is deciding in whether one or the other version string is considered newer.
If a version string has multiple consecutive delimiters between two segments and there are more delimiters present than in its counterpart version string at the same position, the first version string is always considered newer.

**NOTE**: The reason for this behavior is unclear and it is not certain whether it has been added intentionally.

Examples:

```text
1...0 > 1.2
```

For `1...0` and `1.2`, the first segments `1` and `1` are equal.
The second segments `0` and `2` are not equal, but the segment in the first version string is preceded by three dots (`...`) while the other is only preceded by one dot (`.`).
The comparison concludes with `1...0 > 1.2`.
The contents of the second segments are not considered.

#### Differing amounts of segments or sub-segments

Examples:

```text
1 < 1.0
1 < 1.foo
```

The first segments `1` and `1` are equal.
The first version string then ends while the second has another segment.
An additional segment always leads to the version string to be considered newer than the one with no additional segment (this also applies to additional alphanumeric segments).
The comparison concludes with `1 < 1.0` and `1 < 1.foo`.

```text
1.0 > 1.0foo.2
```

The first segments `1` and `1` are equal.
The next sub-segments `0` and `0` are also equal.
The first version string then ends, while the second has another sub-segment `foo`.
As that sub-segment contains only alphabetic characters, it is considered older.
This special case was historically introduced to catch alpha/beta releases that were often marked via a `alpha` suffix _without_ delimiter such as `1.0.0alpha.1`.
However, this is no longer relevant with modern **semantic versioning**.
As a result, the comparison concludes with `1.0 > 1.0foo.2`.
The last segment of `1.0foo.2` is not considered.

```text
1.foo < 1.foo2
```

The first segments `1` and `1` are equal.
The next sub-segments `foo` and `foo` are also equal.
The first version string then ends, while the second has another sub-segment `2`.
As that sub-segment is numerical, it is considered _newer_.
The comparison concludes with `1.foo < 1.foo2`.

#### Version strings with trailing delimiters

Some version strings contain trailing delimiters.
The original **rpmvercmp** implementation explicitly defines comparison on versions with trailing delimiters as _undefined behaviour_.
**vercmp** adopted the algorithm, but never made any statements regarding such edge-cases.
Hence, it is believed that the following rules are not intended behavior and may therefore be subject to change!

For the sake of completeness, the behavior has been analyzed, re-implemented and documented:

Examples:

```text
1... == 1.
```

The first segments `1` and `1` are equal.
Both version strings then end with an arbitrary amount of trailing delimiters.
Counterintuitively, the amount of trailing delimiters **does not** matter in this case and they are always considered equal.
The comparison concludes with `1... == 1.`.

```text
1. > 1.foo.2
```

The first segments `1` and `1` are equal.
The first version string then ends with a delimiter, while the second has another segment `foo`.
For the second segment of the first version, imagine using an empty string.
Because `foo` is alphabetical, it is automatically considered older, although it is not practically compared to another segment.
This logic is a remnant of the special case that was historically introduced to catch alpha/beta releases, which were often marked via a `alpha` suffix such as `1.0.0alpha.1`.
The comparison concludes with `1. > 1.foo.2`.
The remaining segment of `1.foo.2` is not considered.

```text
1. < 1.2
1. < 1.2foo
```

The first segments `1` and `1` are equal.
The first version string then ends with a delimiter, while the second version has a segment/sub-segment `2`.
Because `2` is a numerical value, it is automatically considered newer, as it is not compared to an empty segment.
The comparison concludes with `1. < 1.2` and `1. < 1.2foo`.

```text
1.alpha. < 1.alpha0
```

The next sub-segments `alpha` and `alpha` are also equal.
The first version string then ends with a delimiter, while the second has a sub-segment `0`.
As `0` is numerical, it's considered newer, as it is compared to an empty segment.
The comparison concludes with `1.alpha. < 1.alpha0`.

# SEE ALSO

**BUILDINFO**(5), **PKGBUILD**(5), **PKGINFO**(5), **SRCINFO**(5), **alpm-epoch**(7), **alpm-package-version**(7), **alpm-pkgrel**(7), **vercmp**(8)

# NOTES

1. **rpmvercmp**
   
   <https://github.com/rpm-software-management/rpm/blob/rpm-4.8.1-release/lib/rpmvercmp.c>
1. **semantic versioning**
   
   <https://semver.org/>
