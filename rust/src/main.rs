use std::cmp::max;
use std::hash::Hash;
use std::time::Instant;
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
    let mut total_non_last_time = 0.0;
    let mut total_last_time = 0.0;
    for case in 0..cases {
        let left: Vec<u32> = read_line()
            .split_terminator(" ")
            .map(|s| s.parse().unwrap())
            .collect();
        let right: Vec<u32> = read_line()
            .split_terminator(" ")
            .map(|s| s.parse().unwrap())
            .collect();

        let r = solve(&left, &right, &mut total_non_last_time, &mut total_last_time);

        match r {
            Some(r) => println!("{} {}", case + 1, r),
            None => println!("{} onmogelijk", case + 1),
        }
    }

    println!("total_non_last_time={}", total_non_last_time);
    println!("total_last_time={}", total_last_time);
}

fn solve(left: &[u32], right: &[u32], total_non_last_time: &mut f32, total_last_time: &mut f32) -> Option<u32> {
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

    let try_solve = |max_swaps: Option<u32>| {
        // init
        let mut min_swaps_for: IntMap<u32, u32> = IntMap::default();
        let dummy_min_swaps_for_target = max_swaps.map_or(u32::MAX, |m| m + 1);
        let mut min_swaps_for_target = dummy_min_swaps_for_target;

        min_swaps_for.insert(0, 0);

        let mut rem_sum_left = total_left;
        let mut rem_sum_right = total_right;

        let mut max_map_size = 0;
        let mut got_to_end = false;

        // solver loop
        let start = Instant::now();
        for (i, &(curr_was_right, curr_value)) in enumerate(&rem) {
            if min_swaps_for.is_empty() {
                break;
            }

            max_map_size = max(max_map_size, min_swaps_for.len());

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
                let swaps_left = min_swaps_for_target - swaps;
                let max_swap_amount = swaps_left.saturating_mul(next_value);

                let max_possible_right_to_left = min(max_swap_amount, rem_sum_right);
                if value_left + rem_sum_left + max_possible_right_to_left < target {
                    return;
                }

                // can't reach right target any more
                let max_possible_left_to_right = min(max_swap_amount, rem_sum_left);
                if value_right + rem_sum_right + max_possible_left_to_right < target {
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

            if i == rem.len() - 1 {
                got_to_end = true;
            }
        }

        // println!("max_swaps={:?}, got_to_end={} => max_map_size={}", max_swaps, got_to_end, max_map_size);
        // println!("took {}s", start.elapsed().as_secs_f32());

        if min_swaps_for_target == dummy_min_swaps_for_target {
            None
        } else {
            Some(min_swaps_for_target)
        }
    };

    // let min_swaps_neccessary total_left > total_right {

    // }

    let mut max_swaps = 5;
    let start = Instant::now();
    loop {
        max_swaps += 5;

        let delta = start.elapsed().as_secs_f32();

        let derp = Instant::now();

        if let Some(swaps) = try_solve(Some(max_swaps as u32)) {
            *total_non_last_time += delta;
            *total_last_time += derp.elapsed().as_secs_f32();
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
