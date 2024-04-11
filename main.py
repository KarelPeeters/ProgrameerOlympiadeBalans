import math
import sys
from typing import Dict, List, Optional, Tuple
from ortools.sat.python import cp_model


def solve_dynamic(left, right):
    # prepare stuff
    total = sum(left) + sum(right)
    if total % 2 != 0:
        return None
    target = total // 2

    # print("Problem:")
    # print(f"  left={left}")
    # print(f"  right={right}")
    # print(f"  target={target}")

    rem = [(False, x) for x in left] + [(True, x) for x in right]
    rem.sort(key=lambda e: -e[1])  # TODO try different ways to sort

    # solver step
    min_swaps_for_value_left = {0: 0}
    min_swaps_for_target = math.inf

    rem_sum_left = sum(left)
    rem_sum_right = sum(right)

    for i, (curr_was_right, curr_value) in enumerate(rem):
        # print(f"Iteration {i}, set len {len(min_swaps_for_value_left)}")
        # print(f"# {i}/{len(rem)}", flush=True)
        # assert rem_sum_left == sum(x[1] for x in rem[i:] if not x[0])
        # assert rem_sum_right == sum(x[1] for x in rem[i:] if x[0])

        if len(min_swaps_for_value_left) == 0:
            break

        next_value = rem[i + 1][1] if i + 1 < len(rem) else 0

        rem_sum_left -= (not curr_was_right) * curr_value
        rem_sum_right -= curr_was_right * curr_value

        next_min_swaps_for_value_left = {}

        def add(value, swaps):
            nonlocal min_swaps_for_target
            if swaps >= min_swaps_for_target:
                return

            if value + rem_sum_left == target:
                min_swaps_for_target = min(min_swaps_for_target, swaps)
                return

            max_possible_right_to_left = min(
                (min_swaps_for_target - swaps) * next_value,
                rem_sum_right,
            )
            if target < value:
                return
            if value + rem_sum_left + max_possible_right_to_left < target:
                return

            prev = next_min_swaps_for_value_left.get(value)
            if prev is None or prev > swaps:
                next_min_swaps_for_value_left[value] = swaps

        for v, s in min_swaps_for_value_left.items():
            add(v + (not curr_was_right) * curr_value, s)
            add(v + curr_was_right * curr_value, s + 1)

        min_swaps_for_value_left = next_min_swaps_for_value_left

    if i == len(rem):
        assert rem_sum_right == 0
        assert rem_sum_left == 0

    if min_swaps_for_target == math.inf:
        min_swaps_for_target = None
    return min_swaps_for_target


# TODO cache remaining swaps or best swaps?
#    the first one probably has better collisions and a smaller key
#    TODO: we really only need a cache to stop hitrates
def solve_custom_impl(
    target: int,
    curr_sum_left: int,
    swap_budget: int,
    rem: List[Tuple[bool, int]],
    rem_sum_left: int,
    rem_sum_right: int,
    # TODO full pareto frontier!
    cache: Dict[Tuple[int, int], int],
) -> int:
    # log
    # print(target, curr_sum_left, swap_budget, rem, rem_sum_left, rem_sum)

    # invariant
    # exp_rem_sum_left = sum(x[1] for x in rem if not x[0])
    # exp_rem_sum_right = sum(x[1] for x in rem if x[0])
    # assert (
    #     exp_rem_sum_left == rem_sum_left
    # ), f"Expected {exp_rem_sum_left}, got {rem_sum_left}"
    # assert (
    #     exp_rem_sum_right == exp_rem_sum_right
    # ), f"Expected {exp_rem_sum_right}, got {exp_rem_sum_right}"

    if curr_sum_left + rem_sum_left == target:
        return 0
    if swap_budget == 0 or not rem:
        return None

    # overshot target?
    if target < curr_sum_left:
        return None

    # current value to make swap decision on
    curr_was_right, curr_value = rem[0]
    next_rem = rem[1:]

    # undershot target
    # TODO add flipped bound too, or is that redundant?
    max_possible_right_to_left = min(
        swap_budget * curr_value,
        rem_sum_right,
    )
    if curr_sum_left + rem_sum_left + max_possible_right_to_left < target:
        return None

    # check cache
    cache_key = (curr_sum_left, len(rem))
    cache_value = cache.get(cache_key)
    if cache_value is not None:
        print("cache hit")
        return cache_value

    next_rem_sum_left = rem_sum_left - (not curr_was_right) * curr_value
    next_rem_sum_right = rem_sum_right - (curr_was_right) * curr_value

    # try not swapping first
    # print("keep")
    result_no_swap = solve_custom_impl(
        target=target,
        curr_sum_left=curr_sum_left + (not curr_was_right) * curr_value,
        swap_budget=swap_budget,
        rem=next_rem,
        rem_sum_left=next_rem_sum_left,
        rem_sum_right=next_rem_sum_right,
        cache=cache,
    )
    if result_no_swap is not None:
        swap_budget = min(swap_budget, result_no_swap - 1)

    # try swapping
    # print("swap")
    result_swap = solve_custom_impl(
        target=target,
        curr_sum_left=curr_sum_left + curr_was_right * curr_value,
        swap_budget=swap_budget - 1,
        rem=next_rem,
        rem_sum_left=next_rem_sum_left,
        rem_sum_right=next_rem_sum_right,
        cache=cache,
    )
    if result_swap is not None:
        result_swap += 1

    # return the best option if any
    if result_swap is None:
        result = result_no_swap
    elif result_no_swap is None:
        result = result_swap
    else:
        result = min(result_no_swap, result_swap)

    # update cache
    # TODO also update if result is None, but only us it if the current swap budget is lower or equal
    if result is not None:
        cache[cache_key] = result
    # else:
    #     print(f"failed to write cache with key={cache_key}")

    return result


def solve_custom(left, right):
    # prepare stuff
    total = sum(left) + sum(right)
    if total % 2 != 0:
        return None
    target = total // 2

    rem = [(False, x) for x in left] + [(True, x) for x in right]
    rem.sort(key=lambda e: -e[1])

    # we're about to recurse a lot
    sys.setrecursionlimit(max(sys.getrecursionlimit(), 10 + len(rem)))

    # solve!
    cache = {}
    return solve_custom_impl(
        target=target,
        curr_sum_left=0,
        swap_budget=math.inf,
        rem=rem,
        rem_sum_left=sum(left),
        rem_sum_right=sum(right),
        cache=cache,
    )


def solve_ilp(A, B):
    total = sum(A) + sum(B)
    if total % 2 != 0:
        return None
    target_sum = total // 2

    model = cp_model.CpModel()
    moves = []
    sum_left = []
    sum_right = []

    for i, a in enumerate(A):
        move_ai = model.NewBoolVar(f"move_a{i}")
        moves.append(move_ai)

        sum_left.append(a * (1 - move_ai))
        sum_right.append(a * move_ai)
    for i, b in enumerate(B):
        move_bi = model.NewBoolVar(f"move_b{i}")
        moves.append(move_bi)

        sum_left.append(b * move_bi)
        sum_right.append(b * (1 - move_bi))

    model.Add(sum(sum_left) == target_sum)
    model.Add(sum(sum_right) == target_sum)
    model.Minimize(sum(moves))

    solver = cp_model.CpSolver()
    solver.parameters.num_search_workers = 1
    status = solver.Solve(model)

    if status == cp_model.INFEASIBLE:
        return None
    if status == cp_model.OPTIMAL:
        v = solver.objective_value
        assert int(v) == v
        return int(v)

    assert False, f"Unexpected solver status {status}"


def main():
    assert len(sys.argv) == 2, "Expected single input path"
    path = sys.argv[1]

    with open(path, "r") as f:
        cases = int(f.readline().strip())
        for case in range(cases):
            a = [int(x) for x in f.readline().strip().split()]
            b = [int(x) for x in f.readline().strip().split()]

            # r = solve_ilp(a, b)
            # r = solve_custom(a, b)
            r = solve_dynamic(a, b)

            if r is None:
                print(case + 1, "onmogelijk")
            else:
                print(case + 1, r)
            sys.stdout.flush()


if __name__ == "__main__":
    main()
