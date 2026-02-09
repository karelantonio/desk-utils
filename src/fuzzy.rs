// A (modified?) version of the levenshtein distance, for calculating how similar are two strings
struct Levenshtein<'a> {
    s1: &'a [char],
    s2: &'a [char],
    dp: Vec<i64>,
}

impl<'a> Levenshtein<'a> {
    /// New instance
    fn new(s1: &'a [char], s2: &'a [char]) -> Self {
        Self {
            s1,
            s2,
            dp: vec![-1; s1.len() * s2.len()],
        }
    }

    /// So, the algorithm is as follows:
    /// we have two pointers, `p1` and `p1` each one pointing to some position
    /// in each string, the distance to transform the substrings `S1[p1...]` and `S2[p2..]` to match
    /// can be calculated as follows:
    ///
    /// ```text
    /// dist p1 p1 =
    ///     if p1==len(S1) or p2==len(S2) then
    ///         // Remaining chars
    ///         (len(S1) - p1) + (len(S2) - p2)
    ///     else if S1[p1]==S2[p2]
    ///         // Same char
    ///         dist (p1+1) (p2+1)
    ///     else
    ///         // No distinct, take the minimum of
    ///         // inserting/deleting in S1, in S2 of change one character
    ///         1 + min [dist (p1+1) p2,  dist p1 (p2+1),  dist (p1+1) (p2+1)]
    ///
    /// ans = dist 0 0
    /// ```
    /// The wikipedia article explains a lot better: https://en.wikipedia.org/wiki/Levenshtein_distance
    fn calculate(&mut self) -> usize {
        self.process_actions(0, 0) as usize
    }

    /// Implementation of the algorithm described in [`calculate`] using a dynamic programming approach
    fn process_actions(&mut self, pos1: usize, pos2: usize) -> i64 {
        if pos1 == self.s1.len() || pos2 == self.s2.len() {
            // End of both
            return (self.s1.len() + self.s2.len() - pos1 - pos2) as i64;
        }

        let idx = self.at(pos1, pos2);

        if self.dp[idx] != -1 {
            return self.dp[idx];
        }

        let res = if Self::similar(self.s1[pos1], self.s2[pos2]) {
            self.process_actions(pos1 + 1, pos2 + 1)
        } else {
            // TODO: Do not take into account things like repeated spaces
            self.process_actions(pos1 + 1, pos2)
                .min(self.process_actions(pos1, pos2 + 1))
                .min(self.process_actions(pos1 + 1, pos2 + 1))
                + 1
        };

        self.dp[idx] = res;

        res
    }

    fn similar(c1: char, c2: char) -> bool {
        // No need for it to be fully a equivalence relation lmao
        c1 == c2
            || Self::matches(c1, c2, ' ', '-')
            || Self::matches(c1, c2, ' ', '_')
            || Self::matches(c1, c2, ' ', '\t')
            || Self::matches(c1, c2, '-', '_')
    }

    fn matches(c1: char, c2: char, cc1: char, cc2: char) -> bool {
        c1 == cc1 && c2 == cc2 || c1 == cc2 && c2 == cc1
    }

    /// Get the index at the vector
    fn at(&self, pos1: usize, pos2: usize) -> usize {
        self.s1.len() * pos2 + pos1
    }
}

pub fn search<'a>(term: &str, lst: &[&'a str]) -> Vec<f64> {
    let term: Vec<char> = term.chars().collect();
    let mapped: Vec<Vec<char>> = lst.iter().map(|s| s.chars().collect()).collect();
    let as_slices: Vec<&[char]> = mapped.iter().map(Vec::as_slice).collect();
    search_in_chars(&term, &as_slices)
}

pub fn search_in_chars<'a>(term: &[char], lst: &[&'a [char]]) -> Vec<f64> {
    let mut res = Vec::new();
    for &other in lst {
        let (dist, max) = (
            Levenshtein::new(&term, &other).calculate(),
            term.len().max(other.len()),
        );

        res.push(if max == 0 {
            0f64
        } else {
            dist as f64 / (max as f64)
        });
    }
    res
}

mod tests {
    use crate::fuzzy::Levenshtein;
    use core::iter::Iterator;

    #[test]
    fn basic() {
        assert_eq!(
            Levenshtein::new(
                &"a".chars().collect::<Vec<_>>(),
                &"".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            1
        );

        assert_eq!(
            Levenshtein::new(
                &"".chars().collect::<Vec<_>>(),
                &"b".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            1
        );

        assert_eq!(
            Levenshtein::new(
                &"a".chars().collect::<Vec<_>>(),
                &"b".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            1
        );

        assert_eq!(
            Levenshtein::new(
                &"a".chars().collect::<Vec<_>>(),
                &"a".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            0
        );

        assert_eq!(
            Levenshtein::new(
                &"hello".chars().collect::<Vec<_>>(),
                &"hey".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            3
        );

        assert_eq!(
            Levenshtein::new(
                &"hhello".chars().collect::<Vec<_>>(),
                &"hello".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            1
        );

        assert_eq!(
            Levenshtein::new(
                &"hhello".chars().collect::<Vec<_>>(),
                &"heello".chars().collect::<Vec<_>>(),
            )
            .calculate(),
            1
        );

        assert_eq!(
            Levenshtein::new(
                &"firefox".chars().collect::<Vec<_>>(),
                &"find fox".chars().collect::<Vec<_>>(),
            ) // n->r, r->d,
            .calculate(),
            3
        );

        assert_eq!(
            Levenshtein::new(
                &"firefox-idk".chars().collect::<Vec<_>>(),
                &"firefox_idk".chars().collect::<Vec<_>>(),
            ) // n->r, r->d,
            .calculate(),
            3
        );
    }
}
