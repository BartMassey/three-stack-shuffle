import argparse
import hashlib
import json
import math
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from ternary_merge import ternary_bounds


CUTOFF = 64
COEFFICIENT = 2 / math.log2(3)
DELTA = COEFFICIENT - 1
LEAF_COSTS = (
    (0, 0), (0, 1), (4, 4), (8, 7), (12, 12), (16, 15),
    (20, 20), (24, 23), (28, 28), (36, 39), (40, 44),
    (46, 49), (50, 56), (56, 61), (60, 68), (66, 73),
    (72, 80), (84, 87), (90, 94), (96, 99), (104, 106),
    (110, 111), (118, 118), (124, 125), (132, 132),
    (140, 145), (148, 152), (154, 159), (162, 168),
    (168, 175), (176, 184), (184, 191), (192, 200),
    (200, 209), (206, 218), (214, 225), (220, 234),
    (228, 241), (236, 250), (244, 259), (250, 268),
    (258, 277), (264, 284), (272, 293), (280, 300),
    (288, 309), (296, 318), (304, 327), (312, 334),
    (326, 343), (334, 350), (342, 359), (352, 368),
    (360, 377), (370, 386), (378, 393), (388, 402),
    (398, 409), (408, 418), (416, 427), (426, 436),
    (434, 443), (444, 452), (454, 459), (464, 468),
)


def leaf_shift():
    return min(
        (costs[endpoint] / n - COEFFICIENT * math.log2(n)
         - endpoint * DELTA, n, endpoint)
        for n, costs in enumerate(LEAF_COSTS[1:], 1)
        for endpoint in (0, 1)
    )


def potential(n, endpoint, shift=-8):
    if n == 0:
        return 0.0
    return n * (COEFFICIENT * math.log2(n) + shift + endpoint * DELTA)


def optimized_recurrence(maximum, repark=True):
    if maximum < 0:
        raise ValueError("maximum must be nonnegative")
    central = [costs[0] for costs in LEAF_COSTS[:maximum + 1]]
    parked = [costs[1] for costs in LEAF_COSTS[:maximum + 1]]
    for n in range(CUTOFF + 1, maximum + 1):
        central.append(n + min(parked[k] + parked[n - k]
                               for k in range(1, n)))
        value = n + min(parked[k] + central[n - k] + k
                        for k in range(1, n))
        if repark:
            value = min(value, central[n] + n - 2)
        parked.append(value)
    return central, parked


def report(maximum):
    if maximum < CUTOFF:
        raise ValueError("report maximum must be at least 64")
    shift, limiting_n, limiting_endpoint = leaf_shift()
    result = {
        "model": "fixed endpoint leaf charges; all positive binary splits",
        "cutoff": CUTOFF,
        "maximum": maximum,
        "coefficient": COEFFICIENT,
        "delta": DELTA,
        "proof_shift": -8,
        "best_leaf_shift_numerical": shift,
        "limiting_leaf_numerical": {
            "n": limiting_n,
            "endpoint": "P" if limiting_endpoint else "U",
        },
        "leaf_costs": LEAF_COSTS,
        "variants": {},
    }
    sample_sizes = sorted({n for n in (0, 1, 2, 8, 24, 52, 64, 65,
                                      81, 128, 243, 512, 729, 1024,
                                      2187, 4096, maximum) if n <= maximum})
    upper = [ternary_bounds(n) for n in range(maximum + 1)]
    for repark in (False, True):
        values = optimized_recurrence(maximum, repark=repark)
        assert all(values[endpoint][n] <= upper[n][endpoint]
                   for n in range(maximum + 1) for endpoint in (0, 1))
        assert all(values[endpoint][n] + 1e-8 >= potential(n, endpoint)
                   for n in range(maximum + 1) for endpoint in (0, 1))
        sharp_margin = min(
            (values[endpoint][n] - potential(n, endpoint, shift), n, endpoint)
            for n in range(1, maximum + 1) for endpoint in (0, 1)
        )
        assert sharp_margin[0] >= -1e-8
        payload = json.dumps(values, separators=(",", ":")).encode()
        name = "with_repark" if repark else "without_repark"
        result["variants"][name] = {
            "all_sizes_satisfy_lower_and_ternary_upper": True,
            "minimum_sharp_potential_margin": sharp_margin,
            "cost_arrays_sha256": hashlib.sha256(payload).hexdigest(),
            "hash_format": "json.dumps([central, parked], separators=(',', ':'))",
            "samples": [
                {"n": n, "central": values[0][n], "parked": values[1][n],
                 "ternary_central": upper[n][0], "ternary_parked": upper[n][1],
                 "sharp_lower_central": potential(n, 0, shift),
                 "sharp_lower_parked": potential(n, 1, shift)}
                for n in sample_sizes
            ],
        }
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--maximum", type=int, default=4096)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = report(args.maximum)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"maximum": args.maximum,
                      "coefficient": result["coefficient"],
                      "output": str(args.output)}))


if __name__ == "__main__":
    main()
