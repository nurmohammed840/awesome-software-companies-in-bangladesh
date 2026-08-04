//! Copied From: https://github.com/TheAlgorithms/Rust/blob/master/src/string/levenshtein_distance.rs#L89

use std::cmp::min;

/// Calculates the Levenshtein distance between two strings using an optimized dynamic programming approach.
///
/// This edit distance is defined as 1 point per insertion, substitution, or deletion required to make the strings equal.
///
/// # Arguments
///
/// * `string1` - The first string.
/// * `string2` - The second string.
///
/// # Returns
///
/// The Levenshtein distance between the two input strings.
/// For a detailed explanation, check the example on [Wikipedia](https://en.wikipedia.org/wiki/Levenshtein_distance).
/// This function iterates over the bytes in the string, so it may not behave entirely as expected for non-ASCII strings.
///
/// Note that this implementation utilizes an optimized dynamic programming approach, significantly reducing the space complexity from O(nm) to O(n), where n and m are the lengths of `string1` and `string2`.
///
/// Additionally, it minimizes space usage by leveraging the shortest string horizontally and the longest string vertically in the computation matrix.
///
/// # Complexity
///
/// - Time complexity: O(nm),
/// - Space complexity: O(n),
///
/// where n and m are lengths of `string1` and `string2`.
pub fn levenshtein_distance(string1: &str, string2: &str) -> usize {
    if string1.is_empty() {
        return string2.len();
    }
    let l1 = string1.len();
    let mut prev_dist: Vec<usize> = (0..=l1).collect();

    for (row, c2) in string2.chars().enumerate() {
        // we'll keep a reference to matrix[i-1][j-1] (top-left cell)
        let mut prev_substitution_cost = prev_dist[0];
        // diff with empty string, since `row` starts at 0, it's `row + 1`
        prev_dist[0] = row + 1;

        for (col, c1) in string1.chars().enumerate() {
            // "on the left" in the matrix (i.e. the value we just computed)
            let deletion_cost = prev_dist[col] + 1;
            // "on the top" in the matrix (means previous)
            let insertion_cost = prev_dist[col + 1] + 1;
            let substitution_cost = if c1 == c2 {
                // last char is the same on both ends, so the min_distance is left unchanged from matrix[i-1][i+1]
                prev_substitution_cost
            } else {
                // substitute the last character
                prev_substitution_cost + 1
            };
            // save the old value at (i-1, j-1)
            prev_substitution_cost = prev_dist[col + 1];
            prev_dist[col + 1] = _min3(deletion_cost, insertion_cost, substitution_cost);
        }
    }
    prev_dist[l1]
}

#[inline]
fn _min3<T: Ord>(a: T, b: T, c: T) -> T {
    min(a, min(b, c))
}
