import argparse
from collections import Counter
import itertools
import json
from pathlib import Path
import resource
import sys
from time import monotonic

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.campaign_complexity import active_size, color_schedule, replay
from tools.campaign_complexity import two_chain_plan


def balanced_words(pairs, opened=0, closed=0, prefix=()):
    if closed == pairs:
        yield prefix
    if opened < pairs:
        yield from balanced_words(pairs, opened + 1, closed,
                                  prefix + ("in",))
    if closed < opened:
        yield from balanced_words(pairs, opened, closed + 1,
                                  prefix + ("out",))


def components(word):
    result = []
    start = 0
    depth = 0
    for position, direction in enumerate(word):
        depth += 1 if direction == "in" else -1
        if depth == 0:
            result.append(word[start:position + 1])
            start = position + 1
    return result


def event_schedule(skeleton, pieces, gaps, labels):
    labels = iter(labels)
    inserted = {}
    for piece, gap in zip(pieces, gaps):
        temporary = []
        events = inserted.setdefault(gap, [])
        for direction in piece:
            if direction == "in":
                card = next(labels)
                temporary.append(card)
            else:
                card = temporary.pop()
            events.append((direction, card))
    events = []
    for position, event in enumerate(skeleton):
        events.extend(inserted.get(position, ()))
        events.append(event)
    return events


def central_valid(size, target, events):
    central = list(reversed(range(size)))
    present = set(range(size))
    for direction, card in events:
        if direction == "out":
            if not central or central[-1] != card:
                return False
            central.pop()
            present.remove(card)
        else:
            if card in present:
                return False
            central.append(card)
            present.add(card)
    return tuple(reversed(central)) == target


def bounded_plan(permutation, excess, deadline):
    baseline = two_chain_plan(permutation)
    if baseline is not None:
        return baseline, 0
    size = active_size(permutation)
    target = permutation[:size]
    skeleton = [("out", card) for card in range(size)]
    skeleton += [("in", card) for card in reversed(target)]
    attempts = 0
    for extra in range(1, excess + 1):
        for balanced in balanced_words(extra):
            pieces = components(balanced)
            placements = itertools.combinations_with_replacement(
                range(1, 2 * size), len(pieces))
            for gaps in placements:
                for labels in itertools.product(range(size), repeat=extra):
                    attempts += 1
                    if attempts % 1000 == 0 and monotonic() > deadline:
                        raise TimeoutError("time budget exceeded")
                    events = event_schedule(skeleton, pieces, gaps, labels)
                    if not central_valid(size, target, events):
                        continue
                    word = color_schedule(size, events)
                    if word is not None:
                        assert replay(size, word)[0] == ((), target, ())
                        return word, attempts
    return None, attempts


def run_size(size, max_excess, directory, deadline):
    started = monotonic()
    exact = json.loads((directory / f"exact-n{size}.json").read_text())
    attempts = 0
    yes = Counter()
    decisions = 0
    for rank, target in enumerate(itertools.permutations(range(size))):
        optimum = exact["target_distances"][rank]
        active = active_size(target)
        for excess in range(max_excess + 1):
            word, count = bounded_plan(target, excess, deadline)
            attempts += count
            decisions += 1
            assert (word is not None) == (optimum <= 2 * active + 2 * excess)
            if word is not None:
                assert len(word) == optimum
                assert replay(size, word)[0] == ((), target, ())
                yes[excess] += 1
    return {"n": size, "targets": len(exact["target_distances"]),
            "decisions": decisions, "yes_by_excess": dict(yes),
            "attempted_schedules": attempts,
            "elapsed_seconds": monotonic() - started}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-n", type=int, default=6)
    parser.add_argument("--max-excess", type=int, default=1)
    parser.add_argument("--seconds", type=float, default=50)
    parser.add_argument("--output", type=Path,
                        default=Path("results/parameter-complexity.json"))
    args = parser.parse_args()
    resource.setrlimit(resource.RLIMIT_AS, (512 * 1024**2, 512 * 1024**2))
    root = Path(__file__).resolve().parents[1]
    started = monotonic()
    result = {"algorithm": "fixed skeleton plus balanced temporary blocks",
              "time_bound": "f(r) (m+1)^(2r+2)",
              "max_excess": args.max_excess, "sizes": [],
              "memory_limit_mib": 512,
              "time_limit_seconds": args.seconds}
    for size in range(1, args.max_n + 1):
        record = run_size(size, args.max_excess, root / "results",
                          started + args.seconds)
        result["sizes"].append(record)
        print(json.dumps(record), flush=True)
    result["elapsed_seconds"] = monotonic() - started
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
