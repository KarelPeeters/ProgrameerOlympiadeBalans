use std::cmp::max;
use std::hash::Hash;
use std::time::Instant;
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
    // calculate target
    let total_left: u32 = left.iter().copied().sum();
    let total_right: u32 = right.iter().copied().sum();

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

    // prepare max_swaps
    // max_swaps_sum_ltr: (i, swaps_left) -> int
    // let mut max_swap_amount_ltr = vec![];
    // let mut max_swap_amount_rtl = vec![];

    // for i in (0..rem.len()).rev() {
    //     // this we can probably skip the sorts
    //     let mut rem_left = vec![];
    //     let mut rem_right = vec![];
    //     for &e in &rem[i..] {
    //         if e.0 {
    //             rem_right.push(e.1);
    //         } else {
    //             rem_left.push(e.1);
    //         }
    //     }
    //     rem_left.sort_by_key(|&x| Reverse(x));
    //     rem_right.sort_by_key(|&x| Reverse(x));

    //     max_swap_amount_ltr.push(partial_sums(&rem_left));
    //     max_swap_amount_rtl.push(partial_sums(&rem_right));
    // }

    // max_swap_amount_ltr.reverse();
    // max_swap_amount_rtl.reverse();

    // println!("{:?} {:?}", left, right);

    let left_sorted = left.iter().copied().sorted_by_key(|&x| Reverse(x)).collect_vec();
    let right_sorted = right.iter().copied().sorted_by_key(|&x| Reverse(x)).collect_vec();
    let max_swap_amounts_ltr = partial_sums(&left_sorted);
    let max_swap_amounts_rtl = partial_sums(&right_sorted);

    // println!("rem={:?}", rem);
    // println!("max_swap_amount_ltr={:?}", max_swap_amounts_ltr);
    // println!("max_swap_amount_rtl={:?}", max_swap_amounts_rtl);

    let try_solve = |max_swaps: Option<u32>| {
        // init
        let mut min_swaps_for: IntMap<u32, u32> = IntMap::default();
        let dummy_min_swaps_for_target = max_swaps.map_or(u32::MAX, |m| m + 1);
        let mut min_swaps_for_target = dummy_min_swaps_for_target;

        min_swaps_for.insert(0, 0);

        let mut rem_sum_left = total_left;
        let mut rem_sum_right = total_right;

        let mut rem_left = left.len() as u32;
        let mut rem_right = right.len() as u32;

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
                rem_right -= 1;
            } else {
                rem_sum_left -= curr_value;
                rem_left -= 1;
            }

            // TODO replace with just an extra tiny for loop
            let mut add = |value_left, swaps| {
                // too many steps used?
                if swaps >= min_swaps_for_target {
                    return;
                }

                // reached target
                if value_left + rem_sum_left == target {
                    min_swaps_for_target = min(min_swaps_for_target, swaps);
                    return;
                }

                // overshot left target
                if value_left > target {
                    return;
                }
                // overshot right target
                let value_right = total - value_left - rem_sum_left - rem_sum_right;
                if value_right > target {
                    return;
                }

                // can't reach left target any more
                // let max_swap_amount_rtl = &max_swap_amount_rtl[i + 1];
                // let max_swap_amount_rtl = max_swap_amount_rtl[min(swaps_left as usize, max_swap_amount_rtl.len() - 1)];
                
                
                let rem_swaps = min_swaps_for_target - swaps;
                let rem_swaps_right = min(rem_swaps, rem_right);
                let rem_swaps_left = min(rem_swaps, rem_left);
                
                
                // TODO give up or fix, the current problem is that these are probably incorrect
                // let max_possible_rtl = max_swap_amounts_rtl[rem_right] - max_swap_amounts_rtl[min(rem_swaps as usize, right.len())];
                // let max_possible_ltr = max_swap_amounts_ltr[rem_left] - max_swap_amounts_ltr[min(rem_swaps as usize, left.len())];

                // assert!(max_possible_rtl <= rem_sum_right, "i={i}, swaps={swaps}, max_possible_rtl={max_possible_rtl}, rem_sum_right={rem_sum_right}");
                // assert!(max_possible_ltr <= rem_sum_left);

                // let max_possible_rtl = min(max_swap_amount_rtl, rem_sum_right);
                // if value_left + rem_sum_left + max_possible_rtl < target {
                //     return;
                // }

                // can't reach right target any more
                // let max_swap_amount_ltr = &max_swap_amount_ltr[i + 1];
                // let max_swap_amount_ltr = max_swap_amount_ltr[min(rem_swaps as usize, max_swap_amount_ltr.len() - 1)];
                // let max_possible_ltr = min(max_swap_amount_ltr, rem_sum_left);
                // if value_right + rem_sum_right + max_possible_ltr < target {
                //     return;
                // }

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

        if min_swaps_for_target == dummy_min_swaps_for_target {
            None
        } else {
            Some(min_swaps_for_target)
        }
    };

    let mut max_swaps = 5;
    loop {
        max_swaps += 5;

        if let Some(swaps) = try_solve(Some(max_swaps as u32)) {
            return Some(swaps);
        }

        if max_swaps > max(left.len(), right.len()) {
            break;
        }
    }

    None

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

fn partial_sums(v: &[u32]) -> Vec<u32> {
    let mut c = 0;

    let mut r = vec![c];
    for &x in v {
        c += x;
        r.push(c);
    }
    r
}
