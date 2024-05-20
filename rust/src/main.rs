use std::cmp::{min, Reverse};
use std::fmt::Write;

use itertools::Itertools;

fn main() {
    let args = std::env::args().collect_vec();
    assert_eq!(args.len(), 2, "Expected single input argument");

    let input_path = &args[1];
    let input = std::fs::read_to_string(input_path).unwrap();
    let mut input_lines = input.lines();

    let mut read_line = || {
        input_lines.next().unwrap().trim()
    };

    let cases: u32 = read_line().parse().unwrap();
    let mut result = String::new();

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
            Some(r) => writeln!(&mut result, "{} {}", case + 1, r).unwrap(),
            None => writeln!(&mut result, "{} onmogelijk", case + 1).unwrap(),
        }
    }

    print!("{}", result);
    assert!(input_lines.next().is_none());
}

fn solve(left: &[u32], right: &[u32]) -> Option<u32> {
    // calculate target
    let total_left: u32 = left.iter().copied().sum();
    let total_right: u32 = right.iter().copied().sum();

    // basic checks
    if total_left == total_right {
        return Some(0);
    }
    let total = total_left + total_right;
    if total % 2 != 0 {
        return None;
    }
    let target = total / 2;

    // decision array
    let mut rem = vec![];
    rem.extend(left.iter().map(|&x| (false, x)));
    rem.extend(right.iter().map(|&x| (true, x)));
    rem.sort_by_key(|&(_, x)| Reverse(x));

    // max possible
    let mut max_possible_left_to_right = vec![0];
    let mut max_possible_right_to_left = vec![0];
    for &(curr_was_right, curr_value) in &rem {
        if curr_was_right {
            max_possible_right_to_left.push(max_possible_right_to_left.last().unwrap() + curr_value);
        } else {
            max_possible_left_to_right.push(max_possible_left_to_right.last().unwrap() + curr_value);
        }
    }

    // init
    let capacity = 1000000;
    let mut min_swaps_for: Vec<(u32, u32)> = Vec::with_capacity(capacity);
    let mut next_min_swaps_for: Vec<(u32, u32)> = Vec::with_capacity(capacity);

    // iterate deepening loop
    for max_swaps in 1..=(rem.len() / 2) as u32 {
        let mut rem_sum_left = total_left;
        let mut done_left: u32 = 0;
        let mut done_right: u32 = 0;

        min_swaps_for.clear();
        min_swaps_for.push((0, 0));

        // swap loop
        for &(curr_was_right, curr_value) in &rem {
            if min_swaps_for.is_empty() {
                break;
            }

            assert!(next_min_swaps_for.is_empty());
            if next_min_swaps_for.capacity() < 2 * min_swaps_for.len() + 1 {
                next_min_swaps_for.reserve(2 * min_swaps_for.len() + 1 - next_min_swaps_for.capacity());
            }
            let cap_start = next_min_swaps_for.capacity();

            if curr_was_right {
                done_right += 1;
            } else {
                rem_sum_left -= curr_value;
                done_left += 1;
            }

            let baseline_left_to_right = max_possible_left_to_right[min(done_left as usize, max_possible_left_to_right.len() - 1)];
            let baseline_right_to_left = max_possible_right_to_left[min(done_right as usize, max_possible_right_to_left.len() - 1)];
            let relative_target = target as i32 - rem_sum_left as i32;

            let mut add = |value_left, swaps, skip1: bool, skip2: bool| -> bool {
                // reached target?
                if !skip1 && value_left as i32 == relative_target {
                    return true;
                }
                debug_assert!(value_left as i32 != relative_target);

                // too many swaps used?
                if !skip1 && swaps >= max_swaps {
                    return false;
                }
                debug_assert!(swaps < max_swaps);
                let swaps_remaining = max_swaps - swaps;

                // max possible
                let max_possible_left = value_left + (
                    max_possible_right_to_left[min((done_right + swaps_remaining) as usize, max_possible_right_to_left.len() - 1)]
                        - baseline_right_to_left
                );

                let min_possible_left = value_left as i32 - (
                    max_possible_left_to_right[min((done_left + swaps_remaining) as usize, max_possible_left_to_right.len() - 1)]
                        - baseline_left_to_right
                ) as i32;

                // reachable
                if !skip2 && ((max_possible_left as i32) < relative_target || min_possible_left > relative_target) {
                    return false;
                }
                debug_assert!(!((max_possible_left as i32) < relative_target || min_possible_left > relative_target));

                // push solution
                next_min_swaps_for.push((value_left, swaps));
                false
            };

            // main loop merge
            if curr_was_right {
                let mut a = 0;
                let mut b = 0;

                while a < min_swaps_for.len() {
                    let next_a = min_swaps_for[a].0;
                    let mut next_b = min_swaps_for[b].0 + curr_value;

                    while next_b < next_a {
                        let swaps_b = min_swaps_for[b].1 + 1;
                        if add(next_b, swaps_b, false, false) {
                            return Some(max_swaps);
                        }
                        b += 1;
                        next_b = min_swaps_for[b].0 + curr_value;
                    }
                    if next_b == next_a {
                        let swaps_a = min_swaps_for[a].1;
                        let swaps_b = min_swaps_for[b].1 + 1;
                        if add(next_a, min(swaps_a, swaps_b), true, true) {
                            return Some(max_swaps);
                        }
                        b += 1;
                    } else {
                        let swaps_a = min_swaps_for[a].1;
                        if add(next_a, swaps_a, true, false) {
                            return Some(max_swaps);
                        }
                    }
                    a += 1;
                }
                while b < min_swaps_for.len() {
                    let next_b = min_swaps_for[b].0 + curr_value;
                    let swaps_b = min_swaps_for[b].1 + 1;
                    if add(next_b, swaps_b, false, false) {
                        return Some(max_swaps);
                    }
                    b += 1;
                }
            } else {
                let mut a = 0;
                let mut b = 0;
                while a < min_swaps_for.len() {
                    let next_a = min_swaps_for[a].0;
                    let mut next_b = min_swaps_for[b].0 + curr_value;
                    while next_b < next_a {
                        let swaps_b = min_swaps_for[b].1;
                        if add(next_b, swaps_b, true, false) {
                            return Some(max_swaps);
                        }
                        b += 1;
                        next_b = min_swaps_for[b].0 + curr_value;
                    }
                    if next_b == next_a {
                        let swaps_a = min_swaps_for[a].1 + 1;
                        let swaps_b = min_swaps_for[b].1;
                        if add(next_a, min(swaps_a, swaps_b), true, true) {
                            return Some(max_swaps);
                        }
                        b += 1;
                    } else {
                        let swaps_a = min_swaps_for[a].1 + 1;
                        if add(next_a, swaps_a, false, false) {
                            return Some(max_swaps);
                        }
                    }
                    a += 1;
                }
                while b < min_swaps_for.len() {
                    let next_b = min_swaps_for[b].0 + curr_value;
                    let swaps_b = min_swaps_for[b].1;
                    if add(next_b, swaps_b, true, false) {
                        return Some(max_swaps);
                    }
                    b += 1;
                }
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
            next_min_swaps_for.clear();
        }
    };

    None
}
