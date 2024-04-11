import sys
from ortools.sat.python import cp_model


def solve(A, B):
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
            r = solve(a, b)
            if r is None:
                print(case + 1, "onmogelijk")
            else:
                print(case + 1, r)


if __name__ == "__main__":
    main()
