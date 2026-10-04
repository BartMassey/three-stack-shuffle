import argparse
import json
from pathlib import Path
import random
import time

from solver import ALGORITHMS
from three_stack import Machine, cancel, validate_problem


def shuffled_target(initial, seed=None):
    generator = random.SystemRandom() if seed is None else random.Random(seed)
    target = list(initial)
    for index in range(len(target) - 1, 0, -1):
        chosen = generator.randrange(index + 1)
        target[index], target[chosen] = target[chosen], target[index]
    return target


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--initial", nargs="+", type=int)
    parser.add_argument("--target", nargs="+", type=int)
    parser.add_argument("--seed", type=int)
    parser.add_argument("--algorithm", choices=ALGORITHMS, default="recommended")
    parser.add_argument("--summary", action="store_true")
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()
    if arguments.n < 0:
        parser.error("n must be nonnegative")
    initial = arguments.initial if arguments.initial is not None else list(
        range(1, arguments.n + 1))
    target = arguments.target if arguments.target is not None else shuffled_target(
        initial, arguments.seed)
    try:
        validate_problem(initial, target)
    except ValueError as error:
        parser.error(str(error))
    started = time.perf_counter()
    operations = cancel(ALGORITHMS[arguments.algorithm](initial, target))
    elapsed = time.perf_counter() - started
    machine = Machine(initial)
    machine.run(operations)
    machine.verify(target)
    result = {"n": len(initial), "initial": initial, "target": target,
              "algorithm": arguments.algorithm, "moves": len(operations),
              "planning_seconds": elapsed, "verified": True}
    if not arguments.summary:
        result["operations"] = operations
    output = json.dumps(result, indent=2) + "\n"
    if arguments.output:
        arguments.output.write_text(output)
    print(output)


if __name__ == "__main__":
    main()
