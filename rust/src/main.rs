#![feature(portable_simd)]

use std::cmp::{max, min, Ordering, Reverse};
use std::fmt::Write;
use std::simd::cmp::SimdOrd;
use std::simd::Simd;

use itertools::{enumerate, Itertools};

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
        let mut min_swaps_for: Vec<(u32, u32)> = vec![];
        let dummy_min_swaps_for_target = max_swaps.map_or(u32::MAX, |m| m + 1);
        let mut min_swaps_for_target = dummy_min_swaps_for_target;

        min_swaps_for.push((0, 0));

        let mut rem_sum_left = total_left;
        let mut rem_sum_right = total_right;

        // solver loop
        for (i, &(curr_was_right, curr_value)) in enumerate(&rem) {
            if min_swaps_for.is_empty() {
                break;
            }

            // reallocate to preserve iteration speed
            //   worst case the number of entries doubles, and in practice that turns out to be enough capacity
            let mut next_min_swaps_for = Vec::with_capacity(min_swaps_for.len() * 2);
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

            let mut add = |value_left, swaps| {
                // if let Some((prev_value, _)) = next_min_swaps_for.last().copied() {
                //     assert!(prev_value < value_left);
                // }

                // too many steps used?
                if swaps >= min_swaps_for_target {
                    return;
                }

                // reached target
                if value_left + rem_sum_left == target {
                    min_swaps_for_target = min(min_swaps_for_target, swaps);
                    return;
                }

                // check target reachability
                let swaps_left = min_swaps_for_target - swaps;
                let max_swap_amount = swaps_left.saturating_mul(next_value);

                //   left
                let max_possible_right_to_left = min(max_swap_amount, rem_sum_right);
                let max_possible_left = value_left + rem_sum_left + max_possible_right_to_left;

                //   right
                let value_right = (total - rem_sum_left - rem_sum_right) - value_left;
                let max_possible_left_to_right = min(max_swap_amount, rem_sum_left);
                let max_possible_right = value_right + rem_sum_right + max_possible_left_to_right;

                if max_possible_left < target || max_possible_right < target {
                    return;
                }

                next_min_swaps_for.push((value_left, swaps));
            };

            let mut a = 0;
            let mut b = 0;

            while a + SIMD_WIDTH < min_swaps_for.len() && b + SIMD_WIDTH < min_swaps_for.len() {
                let mut a_keys = Simd::default();
                let mut a_values = Simd::default();
                let mut b_keys = Simd::default();
                let mut b_values = Simd::default();

                for i in 0..SIMD_WIDTH {
                    a_keys[i] = min_swaps_for[a + i].0;
                    a_values[i] = min_swaps_for[a + i].1;
                    b_keys[i] = min_swaps_for[b + i].0;
                    b_values[i] = min_swaps_for[b + i].1;
                }

                a_keys += Simd::splat(if !curr_was_right { curr_value } else { 0 });
                b_keys += Simd::splat(if curr_was_right { curr_value } else { 0 });
                b_values += Simd::splat(1);

                // TODO update simd values based on curr and swap

                let StepResult { keys, values, inc_a, inc_b } = simd_step(a_keys, a_values, b_keys, b_values);
                a += inc_a as usize;
                b += inc_b as usize;

                let keys = keys.to_array();
                let values = values.to_array();

                // TODO simd-based add filtering?
                for i in 0..SIMD_WIDTH {
                    add(keys[i], values[i]);
                }
            }

            // handle non-simd remainder
            while let (Some((prev_a, prev_swaps_a)), Some((prev_b, prev_swaps_b))) = (min_swaps_for.get(a).copied(), min_swaps_for.get(b).copied()) {
                let next_a = prev_a + if !curr_was_right { curr_value } else { 0 };
                let swaps_a = prev_swaps_a;

                let next_b = prev_b + if curr_was_right { curr_value } else { 0 };
                let swaps_b = prev_swaps_b + 1;

                match next_a.cmp(&next_b) {
                    Ordering::Less => {
                        add(next_a, swaps_a);
                        a += 1;
                    }
                    Ordering::Greater => {
                        add(next_b, swaps_b);
                        b += 1;
                    }
                    Ordering::Equal => {
                        add(next_a, min(swaps_a, swaps_b));
                        a += 1;
                        b += 1;
                    }
                }
            }

            // push remaining items
            while let Some((prev_a, swaps_a)) = min_swaps_for.get(a).copied() {
                let next_a = prev_a + if !curr_was_right { curr_value } else { 0 };
                let swaps_a = swaps_a;
                add(next_a, swaps_a);
                a += 1;
            }
            while let Some((prev_b, swaps_b)) = min_swaps_for.get(b).copied() {
                let next_b = prev_b + if curr_was_right { curr_value } else { 0 };
                let swaps_b = swaps_b + 1;
                add(next_b, swaps_b);
                b += 1;
            }

            // check that no reallocations happened
            let cap_end = next_min_swaps_for.capacity();
            assert_eq!(
                cap_start,
                cap_end,
                "Expected no resize, but it did happen for len={}",
                next_min_swaps_for.len()
            );

            min_swaps_for = next_min_swaps_for;
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

const SIMD_WIDTH: usize = 4;

type SimdVec = Simd<u32, SIMD_WIDTH>;

#[derive(Debug, Eq, PartialEq)]
struct StepResult {
    keys: SimdVec,
    values: SimdVec,
    inc_a: u8,
    inc_b: u8,
}

fn simd_step(a_keys: SimdVec, a_values: SimdVec, b_keys: SimdVec, b_values: SimdVec) -> StepResult {
    let mut r = simd_step_slow(a_keys, a_values, b_keys, b_values);
    
    let m = a_keys.simd_min(b_keys);

    r
}

fn simd_step_slow(a_keys: SimdVec, a_values: SimdVec, b_keys: SimdVec, b_values: SimdVec) -> StepResult {
    let a_keys: [u32; SIMD_WIDTH] = a_keys.to_array();
    let a_values: [u32; SIMD_WIDTH] = a_values.to_array();
    let b_keys: [u32; SIMD_WIDTH] = b_keys.to_array();
    let b_values: [u32; SIMD_WIDTH] = b_values.to_array();

    let mut r_keys = [0; SIMD_WIDTH];
    let mut r_values = [0; SIMD_WIDTH];
    let mut a = 0;
    let mut b = 0;

    for i in 0..SIMD_WIDTH {
        let (k, v) = match a_keys[a].cmp(&b_keys[b]) {
            Ordering::Less => {
                a += 1;
                (a_keys[a - 1], a_values[a - 1])
            }
            Ordering::Greater => {
                b += 1;
                (b_keys[b - 1], b_values[b - 1])
            }
            Ordering::Equal => {
                a += 1;
                b += 1;
                (a_keys[a - 1], min(a_values[a - 1], b_values[b - 1]))
            }
        };
        r_keys[i] = k;
        r_values[i] = v;
    }

    StepResult {
        keys: SimdVec::from_array(r_keys),
        values: SimdVec::from_array(r_values),
        inc_a: a as u8,
        inc_b: b as u8,
    }
}

#[cfg(test)]
mod tests {
    use std::simd::Simd;

    use rand::{Rng, SeedableRng};
    use rand::rngs::SmallRng;

    use crate::{simd_step, simd_step_slow, SIMD_WIDTH};

    #[test]
    fn random() {
        let mut rng = SmallRng::seed_from_u64(0);

        let mut n = 0;

        loop {
            let mut a_keys = Simd::from_array(rng.gen());
            let mut a_values = Simd::from_array(rng.gen());
            let mut b_keys = Simd::from_array(rng.gen());
            let mut b_values = Simd::from_array(rng.gen());

            // keep values low for initial debugging
            a_keys %= Simd::splat(16);
            a_values %= Simd::splat(16);
            b_keys %= Simd::splat(16);
            b_values %= Simd::splat(16);

            // a and b must each be strictly increasing (this also implies no duplicates)
            let mut skip = false;
            for i in 0..SIMD_WIDTH - 1 {
                if a_keys[i] >= a_keys[i + 1] || b_keys[i] >= b_keys[i + 1] {
                    skip = true;
                }
            }
            if skip {
                continue;
            }

            println!("a_keys={:?}", a_keys);
            println!("a_values={:?}", a_values);
            println!("b_keys={:?}", b_keys);
            println!("b_values={:?}", b_values);

            let result_slow = simd_step_slow(a_keys, a_values, b_keys, b_values);
            let result = simd_step(a_keys, a_values, b_keys, b_values);

            println!("result={:?}", result);

            assert_eq!(result_slow, result);

            n += 1;
            if n >= 32 {
                break
            }
        }
    }
}