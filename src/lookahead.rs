use std::{collections::VecDeque, iter::FusedIterator};

pub struct Lookahead<const N: usize, I: Iterator> {
    iter: I,
    // TODO: Replace with a fixed size queue.
    lookahead: VecDeque<Option<I::Item>>,
}

impl<const N: usize, I: Iterator> Iterator for Lookahead<N, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lookahead.pop_front() {
            Some(item) => item,
            None => self.iter.next(),
        }
    }
}

impl<const N: usize, I: FusedIterator> FusedIterator for Lookahead<N, I> {}

impl<const N: usize, I: Iterator> Lookahead<N, I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            lookahead: VecDeque::with_capacity(N),
        }
    }

    pub fn lookahead(&mut self, n: usize) -> Option<&I::Item> {
        // TODO: verify the lookahead count statically.
        assert!(n < N, "cannot lookahead more than {N}");

        // If the iterator implements `FusedIterator` we don't need to continue after the iterator
        // yields `None` think about optimizing for this case.
        for _ in 0..(n + 1).saturating_sub(self.lookahead.len()) {
            self.lookahead.push_back(self.iter.next());
        }

        self.lookahead.get(n).unwrap().as_ref()
    }
}

