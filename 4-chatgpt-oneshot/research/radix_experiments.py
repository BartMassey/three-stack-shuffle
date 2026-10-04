import itertools
import random
import statistics


def radix(initial, labels, optimized=True):
    stacks = [[], list(reversed(initial)), []]
    moves = 0
    for bit in range(max(labels).bit_length()):
        order = list(reversed(stacks[1]))
        bits = [(labels[rank] >> bit) & 1 for rank in order]
        if optimized and all(a <= b for a, b in zip(bits, bits[1:])):
            continue
        count = len(order)
        if optimized:
            while count and bits[count - 1]:
                count -= 1
        for _ in range(count):
            rank = stacks[1].pop()
            stacks[2 if (labels[rank] >> bit) & 1 else 0].append(rank)
            moves += 1
        for source in [2, 0]:
            while stacks[source]:
                stacks[1].append(stacks[source].pop())
                moves += 1
    assert list(reversed(stacks[1])) == list(range(len(initial)))
    assert not stacks[0] and not stacks[2]
    return moves


def bounds(labels):
    guaranteed = 0
    expected = 0.0
    for bit in range(max(labels).bit_length()):
        buckets = {}
        for label in labels:
            residue = label % (1 << bit)
            pair = buckets.setdefault(residue, [0, 0])
            pair[(label >> bit) & 1] += 1
        for residue in sorted(buckets, reverse=True):
            zeros, ones = buckets[residue]
            if zeros:
                expected += ones / (zeros + 1)
                break
            guaranteed += ones
            expected += ones
    base = 2 * len(labels) * max(labels).bit_length()
    return base - 2 * guaranteed, base - 2 * expected


def main():
    random.seed(271828)
    for n in range(2, 9):
        values = [radix(p, list(range(n))) for p in itertools.permutations(range(n))]
        print('exhaustive', n, len(values), round(statistics.mean(values), 3), max(values))
    samples = [random.sample(range(52), 52) for _ in range(10000)]
    variants = {
        'ordinary': list(range(52)),
        'gap': list(range(20)) + list(range(32, 64)),
    }
    best = variants['gap'][:]
    score = bounds(best)[1]
    for _ in range(100000):
        occupied = set(best)
        trial = sorted((occupied - {random.choice(best)}) | {random.choice(list(set(range(64)) - occupied))})
        trial_score = bounds(trial)[1]
        if trial_score < score or random.random() < 0.0005:
            best, score = trial, trial_score
    variants['searched'] = best
    for name, labels in variants.items():
        values = [radix(p, labels) for p in samples]
        print(name, 'bounds', bounds(labels), 'mean', statistics.mean(values), 'max', max(values), 'labels', labels)


if __name__ == '__main__':
    main()
