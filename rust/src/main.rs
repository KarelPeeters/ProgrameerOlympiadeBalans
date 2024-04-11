use std::{
    cmp::{min, Reverse},
    fs::File,
    io::{BufRead, BufReader},
};

use intmap::{Entry, IntMap};
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

    let cases: u64 = read_line().parse().unwrap();
    for case in 0..cases {
        let left: Vec<u64> = read_line()
            .split_terminator(" ")
            .map(|s| s.parse().unwrap())
            .collect();
        let right: Vec<u64> = read_line()
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

fn solve(left: &[u64], right: &[u64]) -> Option<u64> {
    // calculate target
    let total_left: u64 = left.iter().copied().sum();
    let total_right: u64 = right.iter().copied().sum();

    let total = total_left + total_right;
    if total % 2 != 0 {
        return None;
    }
    let target = total / 2;

    // decision array
    let mut rem = vec![];
    for (is_right, arr) in [(false, left), (true, right)] {
        for &x in arr {
            assert!(x > 0);
            rem.push((is_right, x));
        }
    }
    rem.sort_by_key(|&(_, x)| Reverse(x));

    // init
    // TODO smaller keys and values for cache locality?
    let mut min_swaps_for: IntMap<u64> = IntMap::default();
    let mut next_min_swaps_for: IntMap<u64> = IntMap::default();
    let mut min_swaps_for_target = u64::MAX;

    min_swaps_for.insert(0, 0);

    let mut rem_sum_left = total_left;
    let mut rem_sum_right = total_right;

    // solver loop
    for (i, &(curr_was_right, curr_value)) in enumerate(&rem) {
        if min_swaps_for.is_empty() {
            break;
        }
        assert!(next_min_swaps_for.is_empty());

        let next_value = rem.get(i + 1).map_or(0, |&(_, x)| x);
        if curr_was_right {
            rem_sum_right -= curr_value;
        } else {
            rem_sum_left -= curr_value;
        }

        // TODO replace with just an extra tiny for loop
        let mut add = |value_left, swaps| {
            if swaps >= min_swaps_for_target {
                return;
            }
            if value_left + rem_sum_left == target {
                min_swaps_for_target = min(min_swaps_for_target, swaps);
                return;
            }

            let max_possible_right_to_left = min(
                (min_swaps_for_target - swaps).saturating_mul(next_value),
                rem_sum_right,
            );
            if target < value_left {
                return;
            }
            if value_left + rem_sum_left + max_possible_right_to_left < target {
                return;
            }

            insert_if_less(&mut next_min_swaps_for, value_left, swaps);
        };

        for (&v, &s) in min_swaps_for.iter() {
            add(v + if !curr_was_right { curr_value } else { 0 }, s);
            add(v + if curr_was_right { curr_value } else { 0 }, s + 1);
        }

        min_swaps_for.clear();
        std::mem::swap(&mut min_swaps_for, &mut next_min_swaps_for);
    }

    if min_swaps_for_target == u64::MAX {
        None
    } else {
        Some(min_swaps_for_target)
    }
    // min_swaps_for.get(&target).copied()
}

fn insert_if_less<V: Ord>(map: &mut IntMap<V>, key: u64, value: V) {
    match map.entry(key) {
        Entry::Occupied(mut entry) => {
            let prev = entry.get_mut();
            if &*prev > &value {
                *prev = value;
            }
        }
        Entry::Vacant(entry) => {
            entry.insert(value);
        }
    }
}
