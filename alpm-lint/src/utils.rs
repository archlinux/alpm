/// Trait for calculating edit distance between two types.
pub(crate) trait EditDistance {
    /// Calculate edit distance between `self` and `other`.
    fn edit_distance(&self, other: &Self) -> usize;
}

impl EditDistance for &[u8] {
    /// Calculate edit distance between `self` and `other` using the Levenshtein distance algorithm.
    fn edit_distance(&self, other: &Self) -> usize {
        let mut dp = vec![vec![0; other.len() + 1]; self.len() + 1];

        for i in 0..=self.len() {
            for j in 0..=other.len() {
                if i == 0 {
                    dp[i][j] = j;
                } else if j == 0 {
                    dp[i][j] = i;
                } else if self[i - 1] == other[j - 1] {
                    dp[i][j] = dp[i - 1][j - 1];
                } else {
                    dp[i][j] = 1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1]);
                }
            }
        }

        dp[self.len()][other.len()]
    }
}

impl EditDistance for String {
    /// Calculate edit distance between `self` and `other`.
    ///
    /// Delegates to the byte slice implementation.
    fn edit_distance(&self, other: &Self) -> usize {
        self.as_bytes().edit_distance(&other.as_bytes())
    }
}

impl EditDistance for &str {
    /// Calculate edit distance between `self` and `other`.
    ///
    /// Delegates to the byte slice implementation.
    fn edit_distance(&self, other: &Self) -> usize {
        self.as_bytes().edit_distance(&other.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("kitten", "sitting", 3)]
    #[case("flaw", "lawn", 2)]
    #[case("intention", "execution", 5)]
    #[case("", "", 0)]
    #[case("a", "", 1)]
    #[case("", "a", 1)]
    fn test_edit_distance(#[case] s1: &str, #[case] s2: &str, #[case] expected: usize) {
        let distance = s1.edit_distance(&s2);
        assert_eq!(distance, expected);
    }
}
