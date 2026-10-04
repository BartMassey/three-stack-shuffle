import argparse
from collections import Counter, deque
import itertools
import json
from pathlib import Path
import resource
from time import monotonic


MOVES = ((0, 1, "AD"), (1, 0, "DA"),
         (1, 2, "DB"), (2, 1, "BD"))


def neighbors(state):
    for source, destination, move in MOVES:
        if state[source]:
            stacks = list(state)
            card = stacks[source][0]
            stacks[source] = stacks[source][1:]
            stacks[destination] = (card,) + stacks[destination]
            yield tuple(stacks), move


def replay(n, word):
    state = ((), tuple(range(n)), ())
    trace = [state]
    for move in word:
        choices = {name: successor
                   for successor, name in neighbors(state)}
        state = choices[move]
        trace.append(state)
    return state, trace


def decode(encoded):
    moves = []
    while encoded > 1:
        moves.append(MOVES[encoded & 3][2])
        encoded >>= 2
    return moves[::-1]


def active_size(permutation):
    size = len(permutation)
    while size and permutation[size - 1] == size - 1:
        size -= 1
    return size


def two_chain_plan(permutation):
    size = active_size(permutation)
    tails = [-1, -1]
    assignments = {}
    for card in permutation[:size]:
        candidates = [side for side in range(2)
                      if tails[side] < card]
        if not candidates:
            return None
        side = max(candidates, key=lambda index: tails[index])
        tails[side] = card
        assignments[card] = side
    departures = ["DA" if assignments[card] == 0 else "DB"
                  for card in range(size)]
    returns = ["AD" if assignments[card] == 0 else "BD"
               for card in reversed(permutation[:size])]
    return departures + returns


def color_schedule(n, events):
    central = list(reversed(range(n)))
    openings = {}
    intervals = []
    for time, (direction, card) in enumerate(events):
        if direction == "out":
            if not central or central[-1] != card:
                return None
            central.pop()
            openings[card] = time
        else:
            central.append(card)
            intervals.append((openings.pop(card), time, card))
    graph = [[] for _ in intervals]
    for first, (left, right, _) in enumerate(intervals):
        for second in range(first):
            other_left, other_right, _ = intervals[second]
            if (left < other_left < right < other_right
                    or other_left < left < other_right < right):
                graph[first].append(second)
                graph[second].append(first)
    colors = [-1] * len(intervals)
    for root in range(len(intervals)):
        if colors[root] != -1:
            continue
        colors[root] = 0
        queue = [root]
        while queue:
            vertex = queue.pop()
            for neighbor in graph[vertex]:
                if colors[neighbor] == colors[vertex]:
                    return None
                if colors[neighbor] == -1:
                    colors[neighbor] = 1 - colors[vertex]
                    queue.append(neighbor)
    word = [None] * len(events)
    for (left, right, _), color in zip(intervals, colors):
        word[left] = "DA" if color == 0 else "DB"
        word[right] = "AD" if color == 0 else "BD"
    return word


def one_extra_plan(permutation, deadline):
    initial = two_chain_plan(permutation)
    if initial is not None:
        return initial, 0
    size = active_size(permutation)
    target = permutation[:size]
    attempts = 0
    for special in range(size):
        ordinary = [("out", card) for card in range(size)
                    if card != special]
        ordinary += [("in", card) for card in reversed(target)
                     if card != special]
        for positions in itertools.combinations(range(2 * size + 2), 4):
            attempts += 1
            if attempts % 1000 == 0 and monotonic() > deadline:
                raise TimeoutError("time budget exceeded")
            special_events = iter((("out", special), ("in", special),
                                   ("out", special), ("in", special)))
            ordinary_events = iter(ordinary)
            events = [next(special_events) if time in positions
                      else next(ordinary_events)
                      for time in range(2 * size + 2)]
            word = color_schedule(size, events)
            if word is not None:
                final, _ = replay(size, word)
                if final == ((), target, ()):
                    return word, attempts
    return None, attempts


def run_one_extra(n, directory, deadline):
    started = monotonic()
    exact = json.loads((directory / f"exact-n{n}.json").read_text())
    yes = 0
    attempted_schedules = 0
    for rank, permutation in enumerate(itertools.permutations(range(n))):
        word, attempts = one_extra_plan(permutation, deadline)
        attempted_schedules += attempts
        optimal = exact["target_distances"][rank]
        size = active_size(permutation)
        assert (word is not None) == (optimal <= 2 * size + 2)
        if word is not None:
            assert len(word) == optimal
            assert replay(n, word)[0] == ((), permutation, ())
            yes += 1
    return {"n": n, "targets": len(exact["target_distances"]),
            "at_most_one_extra_yes": yes,
            "attempted_schedules": attempted_schedules,
            "elapsed_seconds": monotonic() - started}


def run_size(n, directory, deadline):
    started = monotonic()
    pure = {}
    for assignments in itertools.product((0, 2), repeat=n):
        stacks = [(), (), ()]
        for card, side in enumerate(assignments):
            stacks[side] = (card,) + stacks[side]
        pure[tuple(stacks)] = ["DA" if side == 0 else "DB"
                               for side in assignments]
    predecessors = {state: None for state in pure}
    distances = {state: n for state in pure}
    queue = deque(pure)
    while queue:
        state = queue.popleft()
        for successor, move in neighbors(state):
            if successor not in distances:
                distances[successor] = distances[state] + 1
                predecessors[successor] = (state, move)
                queue.append(successor)
        if len(distances) % 1000 == 0 and monotonic() > deadline:
            raise TimeoutError("time budget exceeded")
    exact = json.loads((directory / f"exact-n{n}.json").read_text())
    plans = json.loads((directory / f"plans-n{n}.json").read_text())
    assert len(distances) == exact["state_count"]
    failures = []
    histogram = Counter()
    tight = 0
    active = 0
    for rank, permutation in enumerate(itertools.permutations(range(n))):
        optimal = exact["target_distances"][rank]
        size = active_size(permutation)
        word = two_chain_plan(permutation)
        assert (word is not None) == (optimal == 2 * size)
        if word is not None:
            assert len(word) == optimal
            final, _ = replay(n, word)
            assert final == ((), permutation, ())
            tight += 1
        if size != n:
            continue
        active += 1
        target = ((), permutation, ())
        restricted = distances[target]
        histogram[restricted - optimal] += 1
        assert restricted >= optimal
        if restricted > optimal and len(failures) < 5:
            unrestricted_word = decode(plans["plans"][rank])
            final, trace = replay(n, unrestricted_word)
            assert final == target
            reverse_word = []
            cursor = target
            while predecessors[cursor] is not None:
                previous, move = predecessors[cursor]
                reverse_word.append(move)
                cursor = previous
            restricted_word = pure[cursor] + reverse_word[::-1]
            assert replay(n, restricted_word)[0] == target
            assert len(restricted_word) == restricted
            failures.append({"target": permutation,
                             "optimal": optimal,
                             "drain_first_optimal": restricted,
                             "optimal_word": unrestricted_word,
                             "optimal_trace": trace,
                             "drain_first_word": restricted_word})
    return {"n": n, "states": len(distances),
            "targets": len(exact["target_distances"]),
            "active_targets": active,
            "two_chain_exact_targets": tight,
            "drain_first_penalty_histogram": dict(sorted(histogram.items())),
            "counterexamples": failures,
            "elapsed_seconds": monotonic() - started}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-n", type=int, default=7)
    parser.add_argument("--seconds", type=float, default=50)
    parser.add_argument("--mode", choices=("phase", "excess-one"),
                        default="phase")
    parser.add_argument("--output", type=Path,
                        default=Path("results/campaign-complexity.json"))
    args = parser.parse_args()
    resource.setrlimit(resource.RLIMIT_AS, (512 * 1024**2, 512 * 1024**2))
    started = monotonic()
    root = Path(__file__).resolve().parents[1]
    result = {"model": "A-D-B, unit top transfers, identity to target on D",
              "mode": args.mode,
              "restriction": ("Before the first return to D, drain D completely"
                              if args.mode == "phase" else
                              "At most one excursion beyond one per active card"),
              "comparison_domain": ("targets with no common identity bottom suffix"
                                    if args.mode == "phase" else
                                    "all targets, after common suffix trimming"),
              "memory_limit_mib": 512, "time_limit_seconds": args.seconds,
              "sizes": []}
    for n in range(1, args.max_n + 1):
        runner = run_size if args.mode == "phase" else run_one_extra
        record = runner(n, root / "results", started + args.seconds)
        result["sizes"].append(record)
        print(json.dumps({key: value for key, value in record.items()
                          if key != "counterexamples"}), flush=True)
    result["elapsed_seconds"] = monotonic() - started
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
