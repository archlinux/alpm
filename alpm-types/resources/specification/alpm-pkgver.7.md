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

## COMPARISON

When comparing **pkgver** values, segments of the version strings are analyzed in sequence, separating numeric and alphabetic sequences. The following rules apply:

1. **Identical Strings**: Two identical **pkgver** values are considered equal.  
   _Example_: "1.0" == "1.0"

2. **Segment Comparison**: Versions are compared segment by segment. Numeric segments are treated as numbers; non-numeric segments are treated as strings.  
   _Example_: "1.2" < "1.10", "1.2" > "1.a"

3. **Leading Zeros**: Leading zeros are ignored in numeric segments.  
   _Example_: "001" == "1"

4. **Length of Numeric Segments**: The segment with more digits is considered greater.  
   _Example_: "10" > "9"

5. **Alpha vs. Numeric Segments**: Numeric segments are always considered greater than alphabetic segments.  
   _Example_: "1.2" > "1.2a"

6. **Handling of Separators**: Fewer separators indicate a lesser value.  
   _Example_: "1.0.0" > "1.0.0-1"

7. **Remaining Characters**: If segments compared so far are identical, the version with remaining characters is considered newer.  
   _Example_: "1.0.0" < "1.0.0-alpha"

8. **Empty Segments**: An empty segment is older than an alphabetic segment.  
   _Example_: "1.0." > "1.0a"

# SEE ALSO

BUILDINFO(5), PKGBUILD(5), alpm-epoch(7), alpm-pkgrel(7), vercmp(8)
