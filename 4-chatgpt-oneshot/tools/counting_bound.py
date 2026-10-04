import argparse
import json
import math
from fractions import Fraction
from pathlib import Path
from time import monotonic


def basic_cumulative(length):
    pairs = length // 2
    return 1 + 3 * (9 ** pairs - 1) // 8


def summarize(n, counts):
    targets = math.factorial(n)
    cumulative = 1
    deficit_sum = 0
    first_cover = None
    cumulative_counts = []
    for length, words in enumerate(counts):
        if length % 2:
            continue
        if length:
            assert words % 2 == 0
            cumulative += words // 2
        cumulative_counts.append([length, str(cumulative)])
        if cumulative >= targets and first_cover is None:
            first_cover = length
        deficit_sum += max(0, targets - cumulative)
    mean = Fraction(2 * deficit_sum, targets)
    return {
        "n": n,
        "target_count": str(targets),
        "maximum_lower_bound": first_cover,
        "mean_lower_bound_numerator": str(mean.numerator),
        "mean_lower_bound_denominator": str(mean.denominator),
        "mean_lower_bound": float(mean),
        "target_cumulative_upper_bounds": cumulative_counts,
    }


def height_counts(n, maximum_length):
    states = {(0, 0, -1): 1}
    counts = [1]
    for _ in range(maximum_length):
        following = {}
        for (a, b, previous), count in states.items():
            candidates = []
            if a:
                candidates.append((a - 1, b, 0))
            if a + b < n:
                candidates.append((a + 1, b, 1))
                candidates.append((a, b + 1, 2))
            if b:
                candidates.append((a, b - 1, 3))
            for state in candidates:
                if previous >= 0 and state[2] == previous ^ 1:
                    continue
                following[state] = following.get(state, 0) + count
        states = following
        counts.append(sum(states.get((0, 0, move), 0)
                          for move in range(4)))
    return counts


def basic_summary(n):
    targets = math.factorial(n)
    length = 0
    while basic_cumulative(length) < targets:
        length += 2
    counts = [1] + [0 if step % 2 else 2 * 3 ** (step - 1)
                    for step in range(1, max(length, 4 * (n - 1)) + 1)]
    result = summarize(n, counts)
    result["four_n_minus_four"] = 4 * (n - 1)
    result["rules_out_universal_four_n_minus_four"] = (
        basic_cumulative(4 * (n - 1)) < targets)
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path,
                        default=Path("results/lower-bounds.json"))
    parser.add_argument("--height-n", type=int, default=52)
    args = parser.parse_args()
    if not 1 <= args.height_n <= 52:
        parser.error("height-n must be between 1 and 52")
    started = monotonic()
    n = args.height_n
    maximum_length = max(4 * (n - 1), 2)
    counts = height_counts(n, maximum_length)
    improved = summarize(n, counts)
    improved["closed_height_word_counts"] = [str(count) for count in counts]
    improved["elapsed_seconds"] = monotonic() - started
    threshold = 2
    while basic_cumulative(4 * (threshold - 1)) >= math.factorial(threshold):
        threshold += 1
    result = {
        "model": "Adjacent single-card stack moves; uniform final permutations; distances even",
        "basic_bound_formula": "1 + 3*(9^(floor(L/2))-1)/8",
        "basic_first_n_ruling_out_universal_four_n_minus_four": threshold,
        "basic": [basic_summary(value) for value in (52, 100, 200, 211, 212)],
        "height_path": improved,
    }
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({
        "threshold": threshold,
        "height_n": n,
        "height_maximum_lower_bound": improved["maximum_lower_bound"],
        "height_mean_lower_bound": improved["mean_lower_bound"],
        "elapsed_seconds": monotonic() - started,
        "basic": [[item["n"], item["maximum_lower_bound"],
                   item["mean_lower_bound"]] for item in result["basic"]],
    }, indent=2))


if __name__ == "__main__":
    main()
