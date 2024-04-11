use std::cmp::max;
use std::hash::Hash;
use std::vec;
use std::{
    cmp::{min, Reverse},
    collections::hash_map::Entry,
    fs::File,
    io::{BufRead, BufReader},
};

use itertools::{enumerate, Itertools};
use nohash::IntMap;

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

    // precalculated swap amounts left
    let mut max_right_to_left_for_index_swaps = vec![];
    for i in 0..rem.len() {
        let mut rem_right = rem[i..].iter().filter(|e| e.0).map(|e| e.1).collect_vec();
        rem_right.sort();
        rem_right.reverse();

        let mut sub = vec![0];
        let mut cum = 0;

        for j in 0..rem_right.len() {
            cum += rem_right[j];
            sub.push(cum);
        }

        max_right_to_left_for_index_swaps.push(sub);
    }
    max_right_to_left_for_index_swaps.push(vec![0]);
    // return None;

    // println!("Problem:");
    // println!("  left={:?}", left);
    // println!("  right={:?}", right);
    // println!("  rem={:?}", rem);
    // println!("  max={:?}", max_right_to_left_for_index_swaps);

    // init
    // TODO smaller keys and values for cache locality?
    let mut min_swaps_for: IntMap<u64, u64> = IntMap::default();
    let mut min_swaps_for_target = u64::MAX;

    min_swaps_for.insert(0, 0);

    let mut rem_sum_left = total_left;
    let mut rem_sum_right = total_right;

    // solver loop
    for (i, &(curr_was_right, curr_value)) in enumerate(&rem) {
        if min_swaps_for.is_empty() {
            break;
        }

        // reallocate to preserve iteration speed
        // worst case the number of entries doubles, so we add some extra margin with "*2.5"
        let cap_target = max(32, min_swaps_for.len() * 5 / 2);
        let mut next_min_swaps_for =
            IntMap::with_capacity_and_hasher(cap_target, Default::default());
        let cap_start = next_min_swaps_for.capacity();

        // let vec_sparsity =  min_swaps_for.len() as f64 / min_swaps_for.keys().copied().max().unwrap() as f64;
        // let map_sparsity = min_swaps_for.len() as f64 / min_swaps_for.capacity() as f64;
        // println!("Iteration i={}, map_len={}, vec_sparsity={}, map_sparsity={}", i, min_swaps_for.len(), vec_sparsity, map_sparsity);

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

            // let old_max_possible_right_to_left = min(
            //     (min_swaps_for_target - swaps).saturating_mul(next_value),
            //     rem_sum_right,
            // );

            let swaps_left = min_swaps_for_target - swaps;
            assert!(swaps_left > 0);
            let inner = &max_right_to_left_for_index_swaps[i+1];
            let max_possible_right_to_left = inner[min(swaps_left as usize, inner.len() - 1)];

            // assert!(
            //     max_possible_right_to_left <= rem_sum_right,
            //     "Sit i={}, swaps_left={} -> {}, rem_sum_right={}",
            //     i+1,
            //     swaps_left,
            //     max_possible_right_to_left,
            //     rem_sum_right
            // );
            // assert!(max_possible_right_to_left <= old_max_possible_right_to_left);

            if target < value_left {
                return;
            }
            if value_left + rem_sum_left + max_possible_right_to_left < target {
                return;
            }

            insert_if_less(&mut next_min_swaps_for, value_left, swaps);
        };

        for (&v, &s) in &min_swaps_for {
            add(v + if !curr_was_right { curr_value } else { 0 }, s);
            add(v + if curr_was_right { curr_value } else { 0 }, s + 1);
        }

        // check that no reallocations happened
        let cap_end = next_min_swaps_for.capacity();
        assert_eq!(
            cap_start,
            cap_end,
            "Expected no resize, but it did happen for len={}",
            next_min_swaps_for.len()
        );

        std::mem::swap(&mut min_swaps_for, &mut next_min_swaps_for);
    }

    if min_swaps_for_target == u64::MAX {
        None
    } else {
        Some(min_swaps_for_target)
    }
    // min_swaps_for.get(&target).copied()
}

fn insert_if_less<K: Hash + nohash::IsEnabled + Eq, V: Ord>(
    map: &mut IntMap<K, V>,
    key: K,
    value: V,
) {
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
