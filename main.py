import math
import sys
from typing import List, Optional, Tuple
from ortools.sat.python import cp_model


# TODO cache remaining swaps or best swaps?
#    the first one probably has better collisions and a smaller key
#    TODO: we really only need a cache to stop hitrates
def solve_custom_impl(
    target: int,
    curr_sum_left: int,
    swap_budget: int,
    rem: List[Tuple[bool, int]],
    rem_sum_left: int,
    rem_sum: int,
) -> int:
    # print(target, curr_sum_left, swap_budget, rem, rem_sum_left, rem_sum)

    # TODO comment out
    expected_rem_sum_left = sum(x[1] for x in rem if not x[0])
    assert (
        expected_rem_sum_left == rem_sum_left
    ), f"Expected {expected_rem_sum_left}, got {rem_sum}"
    assert rem_sum == sum(x[1] for x in rem)

    if curr_sum_left + rem_sum_left == target:
        return 0
    if swap_budget == 0 or not rem:
        return None
    if target < curr_sum_left or curr_sum_left + rem_sum < target:
        return None

    # TODO combine both earlier cuts into one stronger one (N*max(rem))

    # current value to make swap decision on
    curr_was_right, curr_value = rem[0]
    rem = rem[1:]

    # try not swapping first
    # print("keep")
    result_no_swap = solve_custom_impl(
        target=target,
        curr_sum_left=curr_sum_left + (not curr_was_right) * curr_value,
        swap_budget=swap_budget,
        rem=rem,
        rem_sum_left=rem_sum_left - (not curr_was_right) * curr_value,
        rem_sum=rem_sum - curr_value,
    )
    if result_no_swap is not None:
        swap_budget = min(swap_budget, result_no_swap - 1)

    # try swapping
    # print("swap")
    result_swap = solve_custom_impl(
        target=target,
        curr_sum_left=curr_sum_left + curr_was_right * curr_value,
        swap_budget=swap_budget - 1,
        rem=rem,
        rem_sum_left=rem_sum_left - (not curr_was_right) * curr_value,
        rem_sum=rem_sum - curr_value,
    )
    if result_swap is not None:
        result_swap += 1

    # return the best option if any
    if result_swap is None:
        return result_no_swap
    if result_no_swap is None:
        return result_swap
    return min(result_no_swap, result_swap)


def solve_custom(left, right):
    # prepare stuff
    total_left = sum(left)
    total = total_left + sum(right)

    if total % 2 != 0:
        return None
    target = total // 2

    rem = [(False, x) for x in left] + [(True, x) for x in right]
    rem.sort(key=lambda e: -e[1])

    # we're about to recurse a lot
    sys.setrecursionlimit(max(sys.getrecursionlimit(), 10 + len(rem)))

    # solve!
    return solve_custom_impl(
        target=target,
        curr_sum_left=0,
        swap_budget=math.inf,
        rem=rem,
        rem_sum_left=total_left,
        rem_sum=total,
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
            r = solve_custom(a, b)
            if r is None:
                print(case + 1, "onmogelijk")
            else:
                print(case + 1, r)


if __name__ == "__main__":
    main()
