use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use std::cmp::min;
use std::fmt::{Debug, Formatter};

use itertools::{enumerate, Itertools};

fn main() {
    let args = std::env::args().collect_vec();
    assert_eq!(args.len(), 2, "Expected single input argument");

    let input_path = &args[1];
    let mut input = BufReader::new(File::open(input_path).unwrap());

    let mut read_line = || {
        let mut s = String::new();
        input.read_line(&mut s).unwrap();
        s.trim().to_owned()
    };

    let cases: u32 = read_line().parse().unwrap();
    for case in 0..cases {
        let left: Vec<u32> = read_line()
            .split_terminator(" ")
            .map(|s| s.parse().unwrap())
            .collect();
        let right: Vec<u32> = read_line()
            .split_terminator(" ")
            .map(|s| s.parse().unwrap())
            .collect();

        let r = solve(&left, &right);

        match r {
            Some(r) => println!("{} {}", case + 1, r),
            None => println!("{} onmogelijk", case + 1),
        }
    }
}

fn solve(left: &[u32], right: &[u32]) -> Option<u32> {
    // println!("left={:?}, right={:?}", left, right);

    // calculate target
    let total_left: u32 = left.iter().copied().sum();
    let total_right: u32 = right.iter().copied().sum();

    let total = total_left + total_right;
    if total % 2 != 0 {
        return None;
    }
    let target = total / 2;

    // println!("target={}", target);

    // TODO max swap count that increases over time
    // TODO implicit max swaps, if value reachable through lower sets don't count, and don't bump is empty
    // TODO use highest set bit for bitset len
    let max_swaps = 100;

    // TODO leak memory instead of dropping

    // let mut best = None;

    // swap_count -> reachable values
    let mut prev = {
        let mut initial_set = BitSet::new(1);
        initial_set.set(0, true);
        vec![initial_set]
    };

    // println!("initial {:?}", prev);

    // left
    for &x in left {
        let next_max_swaps = min(max_swaps, prev.len());
        let mut next = vec![BitSet::new(prev[0].len + x as usize); next_max_swaps + 1];

        for (prev_steps, prev_reach) in enumerate(&prev) {
            // no swap (add to left value)
            next[prev_steps].or_assign_shift_up(prev_reach, x);

            if prev_steps + 1 <= next_max_swaps {
                // swap (add nothing to left value)
                next[prev_steps + 1].or_assign_shift_up(prev_reach, 0);
            }
        }
        prev = next;

        // println!("after left {x}:");
        for (s, p) in enumerate(&prev) {
            // println!("  swaps={s}: {:?}", p);
        }
    }

    // right
    for &x in right {
        let next_max_swaps = min(max_swaps, prev.len());
        let mut next = vec![BitSet::new(prev[0].len + x as usize); next_max_swaps + 1];

        for (prev_steps, prev_reach) in enumerate(&prev) {
            // no swap (add nothing to left value)
            next[prev_steps].or_assign_shift_up(prev_reach, 0);

            if prev_steps + 1 <= next_max_swaps {
                // swap (add to left value)
                next[prev_steps + 1].or_assign_shift_up(prev_reach, x);
            }
        }
        prev = next;

        // println!("after right {x} {:?}", prev);
        for (s, p) in enumerate(&prev) {
            // println!("  swaps={s}: {:?}", p);
        }
    }

    // range
    // // println!("range {}..{}", 0, total);

    for (steps, reachable) in enumerate(&prev) {
        if (target as usize) < reachable.len && reachable.get(target as usize) {
            return Some(steps as u32);
        }
    }

    None
}

type Block = u64;

#[derive(Clone, Eq, PartialEq)]
struct BitSet {
    // TODO use 128?
    blocks: Vec<Block>,
    len: usize,
}

impl BitSet {
    fn new(len: usize) -> Self {
        // TODO use calloc?
        let block_count = (len + (Block::BITS as usize - 1)) / Block::BITS as usize;
        BitSet { len, blocks: vec![0; block_count] }
    }

    fn new_with(len: usize, values: &[usize]) -> Self {
        let mut set = BitSet::new(len);
        for &v in values {
            set.set(v, true);
        }
        set
    }

    fn set(&mut self, index: usize, value: bool) {
        let (i, b) = split_index(index);
        // TODO unsafe access
        let block = &mut self.blocks[i];

        if value {
            *block |= 1 << b;
        } else {
            *block &= !(1 << b);
        }
    }

    fn get(&self, index: usize) -> bool {
        let (i, b) = split_index(index);
        // TODO unsafe access
        (self.blocks[i] >> b) & 1 != 0
    }

    // TODO use at least full words
    // TODO expand to simd
    // run the operation (self |= other << shift_up)
    fn or_assign_shift_up(&mut self, other: &Self, shift_up: u32) {
        // println!("or_assign_shift_up: self.len={} other.len={} shift_up={}", self.len, other.len, shift_up);
        assert!(other.len + shift_up as usize <= self.len);

        let (w, b) = split_index(shift_up as usize);
        let limit = min(self.blocks.len(), other.blocks.len() + w + (b != 0) as usize);

        if b == 0 {
            for i in w..limit {
                // println!("  full [{}] |= [{}]", i, i - w);
                self.blocks[i] |= other.blocks[i - w];
            }
        } else {
            for i in w..limit {
                // println!("  partial [{}] |= [{}] >> (32-b) | [{}] << b", i, (i-w) as isize-1, i-w);

                let lower = if i == w { 0 } else { other.blocks[i - w - 1] };
                let higher = other.blocks.get(i - w).copied().unwrap_or(0);
                self.blocks[i] |= (lower >> (Block::BITS - b)) | (higher << b);
            }
        }
    }

    fn or_assign_shift_up_slow(&mut self, other: &Self, shift_up: u32) {
        assert!(other.len + shift_up as usize <= self.len);

        for i in 0..other.len {
            if other.get(i) {
                self.set(i + shift_up as usize, true);
            }
        }
    }
}

fn split_index(index: usize) -> (usize, u32) {
    (index / Block::BITS as usize, (index % Block::BITS as usize) as u32)
}

impl Debug for BitSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let values = (0..self.len).filter(|&i| self.get(i)).collect_vec();
        f.debug_struct("BitSet")
            .field("len", &self.len)
            .field("values", &values).finish()
    }
}

#[cfg(test)]
mod test {
    use rand::{Rng, SeedableRng};
    use rand::rngs::SmallRng;

    use crate::BitSet;

    fn test_shift(left: &BitSet, right: &BitSet, shift: u32) {
        // println!("test_shift");
        // println!("  left before={:?}", left);
        // println!("  right before={:?}", right);
        // println!("  shift={}", shift);

        let mut expected = left.clone();
        expected.or_assign_shift_up_slow(&right, shift);
        // println!("  expected={:?}", expected);

        let mut actual = left.clone();
        actual.or_assign_shift_up(&right, shift);
        // println!("  actual={:?}", actual);

        assert_eq!(expected, actual);
    }

    #[test]
    fn base() {
        let a = BitSet::new(4 + 64);
        let mut b = BitSet::new(4);
        b.set(0, true);
        test_shift(&a, &b, 0);
        test_shift(&a, &b, 1);
        test_shift(&a, &b, 2);
        test_shift(&a, &b, 64);
    }

    #[test]
    fn small() {
        let mut a = BitSet::new(4);
        a.set(0, true);
        a.set(2, true);
        a.set(3, true);
        let mut b = BitSet::new(3);
        b.set(1, true);
        test_shift(&a, &b, 1);
    }

    #[test]
    fn over() {
        let a = BitSet::new_with(133, &[]);
        let b = BitSet::new_with(7, &[0, 2, 3, 4]);
        test_shift(&a, &b, 126);
    }

    #[test]
    fn random() {
        let steps = 1024;
        let max_size = 1024*1024;
        let max_shift = 1024;

        let mut rng = SmallRng::seed_from_u64(0);

        for _ in 0..steps {
            let shift = rng.gen_range(0..=max_shift);

            let len = rng.gen_range(0..max_size);
            let mut a = BitSet::new(len + shift as usize);
            let mut b = BitSet::new(len);

            for i in 0..a.len {
                a.set(i, rng.gen());
            }
            for i in 0..b.len {
                b.set(i, rng.gen());
            }

            test_shift(&a, &b, shift);
        }
    }
}
