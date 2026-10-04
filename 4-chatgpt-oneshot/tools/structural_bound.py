import argparse
import bisect
import itertools
import json
import math
from fractions import Fraction
from pathlib import Path
from time import monotonic


def shape(permutation):
    rows = []
    for value in permutation:
        for row in rows:
            position = bisect.bisect_left(row, value)
            if position == len(row):
                row.append(value)
                break
            row[position], value = value, row[position]
        else:
            rows.append([value])
    return tuple(len(row) for row in rows)


def independent_i2(permutation):
    states = {(-1, -1): 0}
    for value in permutation:
        following = states.copy()
        for (first, second), count in states.items():
            if value > first:
                key = tuple(sorted((value, second)))
                following[key] = max(following.get(key, 0), count + 1)
            if value > second:
                key = (first, value)
                following[key] = max(following.get(key, 0), count + 1)
        states = following
    return max(states.values())


def common_suffix(permutation):
    n = len(permutation)
    suffix = 0
    while suffix < n and permutation[n - suffix - 1] == n - suffix - 1:
        suffix += 1
    return suffix


def instance_bound(permutation):
    return (4 * len(permutation) - 2 * sum(shape(permutation)[:2])
            - 2 * common_suffix(permutation))


def partitions(total, maximum=None):
    if total == 0:
        yield ()
        return
    if maximum is None:
        maximum = total
    for first in range(min(total, maximum), 0, -1):
        for rest in partitions(total - first, first):
            yield (first,) + rest


def tableau_count(partition, factorial):
    column_heights = [sum(length > column for length in partition)
                      for column in range(partition[0])]
    product = 1
    for row, length in enumerate(partition):
        for column in range(length):
            product *= length - column + column_heights[column] - row - 1
    quotient, remainder = divmod(factorial, product)
    assert remainder == 0
    return quotient


def exact_ensemble(n, max_seconds=60):
    started = monotonic()
    factorial = math.factorial(n)
    weight_sum = 0
    i2_sum = 0
    partition_count = 0
    for partition in partitions(n):
        count = tableau_count(partition, factorial)
        weight = count * count
        weight_sum += weight
        i2_sum += sum(partition[:2]) * weight
        partition_count += 1
        if partition_count % 1000 == 0:
            if monotonic() - started > max_seconds:
                raise TimeoutError("partition calculation exceeded its budget")
    assert weight_sum == factorial
    suffix_sum = sum(math.factorial(n - length)
                     for length in range(1, n + 1))
    bound = Fraction(4 * n * factorial - 2 * i2_sum - 2 * suffix_sum,
                     factorial)
    return {
        "n": n,
        "partition_count": partition_count,
        "target_count": str(factorial),
        "tableau_square_weight_sum": str(weight_sum),
        "i2_sum_over_targets": str(i2_sum),
        "suffix_sum_over_targets": str(suffix_sum),
        "mean_lower_bound_numerator": str(bound.numerator),
        "mean_lower_bound_denominator": str(bound.denominator),
        "mean_lower_bound": float(bound),
        "elapsed_seconds": monotonic() - started,
    }


def check_exact_tables(through, results_directory):
    checks = []
    for n in range(1, through + 1):
        data = json.loads((results_directory / f"exact-n{n}.json").read_text())
        assert len(data["target_distances"]) == math.factorial(n)
        bound_sum = 0
        i2_sum = 0
        suffix_sum = 0
        largest_gap = 0
        for permutation, distance in zip(
                itertools.permutations(range(n)), data["target_distances"]):
            i2 = sum(shape(permutation)[:2])
            if n <= 6:
                assert i2 == independent_i2(permutation)
            suffix = common_suffix(permutation)
            active = tuple(value for value in permutation if value < n - suffix)
            assert i2 == sum(shape(active)[:2]) + suffix
            lower = 4 * n - 2 * i2 - 2 * suffix
            assert 0 <= lower <= distance, (n, permutation, lower, distance)
            bound_sum += lower
            i2_sum += i2
            suffix_sum += suffix
            largest_gap = max(largest_gap, distance - lower)
        ensemble = exact_ensemble(n)
        assert i2_sum == int(ensemble["i2_sum_over_targets"])
        assert suffix_sum == int(ensemble["suffix_sum_over_targets"])
        assert Fraction(bound_sum, math.factorial(n)) == Fraction(
            int(ensemble["mean_lower_bound_numerator"]),
            int(ensemble["mean_lower_bound_denominator"]))
        assert instance_bound(tuple(reversed(range(n)))) == 4 * (n - 1)
        checks.append({"n": n, "mean_bound": bound_sum / math.factorial(n),
                       "largest_gap": largest_gap,
                       "checked_targets": math.factorial(n)})
    return checks


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--check-through", type=int, default=0)
    parser.add_argument("--max-seconds", type=float, default=60)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if not 1 <= args.n <= 52:
        parser.error("--n must be between 1 and 52")
    if not 0 <= args.check_through <= 9:
        parser.error("--check-through must be between 0 and 9")
    if args.max_seconds <= 0:
        parser.error("--max-seconds must be positive")
    root = Path(__file__).resolve().parent.parent
    checks = check_exact_tables(args.check_through, root / "results")
    result = exact_ensemble(args.n, args.max_seconds)
    result["model"] = "Adjacent single-card moves on A-D-B; endpoints all on D"
    result["instance_bound_formula"] = "4*n - 2*I2(target) - 2*common_bottom_suffix"
    result["validation"] = checks
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
