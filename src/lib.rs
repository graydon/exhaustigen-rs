// This crate is mostly copyright Aleksey Kladov <aleksey.kladov@gmail.com> with
// a little bit from myself, Graydon Hoare <graydon@pobox.com>. It is licensed
// under MIT + ASL2.0 terms.

pub struct Gen {
    started: bool,
    v: Vec<(usize, usize)>,
    p: usize,
}

impl Gen {
    pub fn new() -> Gen {
        Gen {
            started: false,
            v: Vec::new(),
            p: 0,
        }
    }

    /// Returns `true` when every range of values implied by calls to `gen` is
    /// finished. Otherwise restarts the innermost incomplete range, continuing
    /// the exhaustive scan. This method should be called in the head of a
    /// `while` loop enclosing the test you wish to repeat exhaustively.
    pub fn done(&mut self) -> bool {
        if !self.started {
            self.started = true;
            return false;
        }

        for i in (0..self.v.len()).rev() {
            if self.v[i].0 < self.v[i].1 {
                self.v[i].0 += 1;
                self.v.truncate(i + 1);
                self.p = 0;
                return false;
            }
        }
        true
    }

    /// Returns a value (eventually every value) between 0 and `bound`
    /// inclusive. Every other value-generating method in this type ultimately
    /// funnels into this method, which is responsible (in concert with `done`)
    /// for opening ang stepping through ranges of the generator's state-space.
    pub fn gen(&mut self, bound: usize) -> usize {
        if self.p == self.v.len() {
            self.v.push((0, 0));
        }
        self.p += 1;
        self.v[self.p - 1].1 = bound;
        self.v[self.p - 1].0
    }

    // All remaining methods are just helper utilities.

    /// Returns false, then true.
    pub fn flip(&mut self) -> bool {
        self.gen(1) == 1
    }

    /// Selects an element (eventually every element) from `input`.
    pub fn pick<'a, T>(&mut self, input: &'a [T]) -> &'a T {
        &input[self.gen(input.len() - 1)]
    }

    /// Generates a variable-length iterator (eventually every such iterator)
    /// that returns the result of repeated calls to `f(gen)`. The iterator has
    /// length <= `bound`.
    pub fn gen_bound_by<'clo: 'gen, 'gen, T: 'gen, F: 'clo + FnMut(&mut Self) -> T>(
        &'gen mut self,
        bound: usize,
        f: F,
    ) -> impl Iterator<Item = T> + 'gen {
        let fixed = self.gen(bound);
        self.gen_fixed_by(fixed, f)
    }

    /// Generates a fixed-length iterator (eventually every such iterator) that
    /// returns the result of repeated calls to `f(gen)`. The iterator has
    /// length == `fixed`.
    pub fn gen_fixed_by<'clo: 'gen, 'gen, T, F: 'clo + FnMut(&mut Self) -> T>(
        &'gen mut self,
        fixed: usize,
        mut f: F,
    ) -> impl Iterator<Item = T> + 'gen {
        std::iter::repeat_with(move || f(self)).take(fixed)
    }

    /// Generates a variable-length iterator (eventually every such iterator)
    /// with variable-value elements. The iterator has length <= `len_bound` and
    /// each element has value <= `elt_bound`.
    pub fn gen_elts(
        &mut self,
        len_bound: usize,
        elt_bound: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.gen_bound_by(len_bound, move |g| g.gen(elt_bound))
    }

    /// Generates a variable-size combination (eventually every combination) of
    /// elements selected from the `input` array provided, up to the size of
    /// that array. Equivalent to `gen_comb_bound(input, input.len())`.
    pub fn gen_comb<'gen, 'data: 'gen, T>(
        &'gen mut self,
        input: &'data [T],
    ) -> impl Iterator<Item = &'data T> + 'gen {
        let bound = input.len();
        self.gen_bound_comb(bound, input)
    }

    /// Generates a variable-size combination (eventually every combination) of
    /// elements selected from the `input` array provided. Equivalent to
    /// `gen_comb_fixed(input, self.gen(bound))`.
    pub fn gen_bound_comb<'gen, 'data: 'gen, T>(
        &'gen mut self,
        bound: usize,
        input: &'data [T],
    ) -> impl Iterator<Item = &'data T> + 'gen {
        let fixed = self.gen(bound);
        self.gen_fixed_comb(fixed, input)
    }

    /// Generates a fixed-size combination (eventually every combination) of
    /// elements selected from the `input` array provided. In other words,
    /// returns an iterator that produces a sequence of length exactly equal to
    /// `bound` where each element is an independent call to `self.pick(input)`,
    /// and may therefore repeat elements.
    pub fn gen_fixed_comb<'gen, 'data: 'gen, T>(
        &'gen mut self,
        fixed: usize,
        input: &'data [T],
    ) -> impl Iterator<Item = &'data T> + 'gen {
        self.gen_fixed_by(fixed, move |g| g.pick(input))
    }

    /// Generates a permutation (eventually every permutation) of the `input` array
    /// provided. In other words, returns an iterator that produces the exact same set
    /// of items that are in `input` but in some unspecified (eventually every) order.
    pub fn gen_perm<'gen, 'data: 'gen, T>(
        &'gen mut self,
        input: &'data [T],
    ) -> impl Iterator<Item = &'data T> + 'gen {
        let mut idxs = (0..input.len()).collect::<Vec<_>>();
        self.gen_fixed_by(input.len(), move |g| {
            &input[idxs.remove(g.gen(idxs.len() - 1))]
        })
    }

    /// Generates a subset (eventually every subset) of the `input` array provided.
    /// In other words, returns an iterator that calls `self.flip()` to decide
    /// whether to include each element of the input.
    pub fn gen_subset<'gen, 'data: 'gen, T>(
        &'gen mut self,
        input: &'data [T],
    ) -> impl Iterator<Item = &'data T> + 'gen {
        (0..input.len()).filter_map(move |i| if self.flip() { Some(&input[i]) } else { None })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_elts() {
        let mut gen = Gen::new();
        let mut i = 0;
        while !gen.done() {
            let elts = gen.gen_elts(3, 4).collect::<Vec<_>>();
            println!("{:?}", elts);
            i += 1;
        }
        assert_eq!(i, (5 * 5 * 5) + (5 * 5) + 5 + 1);
    }

    #[test]
    fn test_comb() {
        let mut gen = Gen::new();
        let vec = vec![1, 2, 3, 4, 5];
        let mut i = 0;
        while !gen.done() {
            let comb = gen.gen_comb(&vec).collect::<Vec<_>>();
            println!("{:?}", comb);
            i += 1;
        }
        assert_eq!(
            i,
            (5 * 5 * 5 * 5 * 5) + (5 * 5 * 5 * 5) + (5 * 5 * 5) + (5 * 5) + 5 + 1
        );
    }

    #[test]
    fn test_perm() {
        let mut gen = Gen::new();
        let vec = vec![1, 2, 3, 4, 5];
        let mut i = 0;
        while !gen.done() {
            let perm = gen.gen_perm(&vec).collect::<Vec<_>>();
            println!("{:?}", perm);
            i += 1;
        }
        assert_eq!(i, 5 * 4 * 3 * 2 * 1);
    }

    #[test]
    fn test_subset() {
        let mut gen = Gen::new();
        let vec = vec![1, 2, 3, 4, 5];
        let mut i = 0;
        while !gen.done() {
            let subset = gen.gen_subset(&vec).collect::<Vec<_>>();
            println!("{:?}", subset);
            i += 1;
        }
        assert_eq!(i, 1 << 5);
    }
}
