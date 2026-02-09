struct Levenshtein<'a> {
    s1: &'a [char],
    s2: &'a [char],
    dp: Vec<i64>,
}

impl<'a> Levenshtein<'a> {
    fn new(s1: &'a [char], s2: &'a [char]) -> Self {
        Self {
            s1,
            s2,
            dp: vec![-1; s1.len() * s2.len()],
        }
    }

    fn calculate(&mut self) -> usize {
        self.process_actions(0, 0) as usize
    }

    fn process_actions(&mut self, pos1: usize, pos2: usize) -> i64 {
        if pos1 == self.s1.len() || pos2 == self.s2.len() {
            // End of both
            return (self.s1.len() + self.s2.len() - pos1 - pos2) as i64;
        }

        let idx = self.at(pos1, pos2);

        if self.dp[idx] != -1 {
            return self.dp[idx];
        }

        if self.s1[pos1] == self.s2[pos2] {
            return self.process_actions(pos1 + 1, pos2 + 1);
        }

        return self
            .process_actions(pos1 + 1, pos2)
            .min(self.process_actions(pos1, pos2 + 1))
            .min(self.process_actions(pos1 + 1, pos2 + 1))
            + 1;
    }

    /// Get the index at the vector
    fn at(&self, pos1: usize, pos2: usize) -> usize {
        self.s1.len() * pos2 + pos1
    }
}

pub fn search<'a>(term: &str, lst: &[&'a str]) -> Vec<f64> {
    todo!()
}

mod tests {
    use core::iter::Iterator;

    use crate::fuzzy::Levenshtein;

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
    }
}
