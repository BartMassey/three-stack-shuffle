import argparse
import json
from pathlib import Path
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from benchmark import benchmark
from solver import ALGORITHMS
from three_stack import Machine


def structures(n):
    initial = list(range(n))
    pairs = initial[:]
    for index in range(0, n - 1, 2):
        pairs[index], pairs[index + 1] = pairs[index + 1], pairs[index]
    half = n // 2
    interleaved = []
    for index in range(half):
        interleaved.extend((index, half + index))
    interleaved.extend(initial[2 * half:])
    blocks = []
    for start in range(0, n, 8):
        blocks.extend(initial[start:start + 8][::-1])
    return {"identity": initial, "reversal": initial[::-1],
            "rotate_one": initial[1:] + initial[:1],
            "rotate_half": initial[half:] + initial[:half],
            "adjacent_swaps": pairs, "interleaved_halves": interleaved,
            "reversed_blocks_of_eight": blocks}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--suite", choices=("scaling", "structures"), required=True)
    parser.add_argument("--samples", type=int, default=20)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    if arguments.samples < 1:
        parser.error("samples must be positive")
    if arguments.suite == "scaling":
        result = [benchmark(n, arguments.samples, 20261003,
                            ["fast", "recommended"])
                  for n in (1, 2, 4, 8, 9, 16, 32, 52, 64, 65,
                            128, 256, 1024, 4096)]
    else:
        result = []
        initial = list(range(52))
        for name, target in structures(52).items():
            entry = {"structure": name, "target": target, "algorithms": {}}
            for algorithm_name in ("fast", "recommended", "thorough"):
                started = time.perf_counter()
                operations = ALGORITHMS[algorithm_name](initial, target)
                elapsed = time.perf_counter() - started
                machine = Machine(initial)
                machine.run(operations)
                machine.verify(target)
                entry["algorithms"][algorithm_name] = {
                    "moves": len(operations), "planning_seconds": elapsed}
            result.append(entry)
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(json.dumps(result, indent=2) + "\n")
    for entry in result:
        print(entry.get("n", entry.get("structure")),
              {name: data.get("mean", data.get("moves"))
               for name, data in entry["algorithms"].items()})


if __name__ == "__main__":
    main()
