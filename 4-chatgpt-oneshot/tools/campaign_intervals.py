import argparse
import json
from pathlib import Path
import statistics
import sys
from time import monotonic

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from solver import parked
from three_stack import Machine


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path,
                        default=Path("results/campaign-structure-52-paired.json"))
    parser.add_argument("--output", type=Path,
                        default=Path("results/campaign-intervals-52.json"))
    args = parser.parse_args()
    source = json.loads(args.input.read_text())
    rows = []
    for item in source["cases"]:
        target = item["target"]
        initial = list(range(len(target)))
        started = monotonic()
        word = parked(initial, target)
        seconds = monotonic() - started
        machine = Machine(initial)
        machine.run(word)
        machine.verify(target)
        lower = item["certified_lower_bound"]
        assert lower <= len(word) <= 410
        rows.append({"target": target, "lower": lower, "upper": len(word),
                     "structural_bound": item["structural_bound"],
                     "lower_search_interrupted": item["halted"] is not None,
                     "upper_word": word, "planning_seconds": seconds})
    status = Path("/proc/self/status").read_text()
    high_water = next(line.split(":", 1)[1].strip() for line in status.splitlines()
                      if line.startswith("VmHWM:"))
    summary = {"count": len(rows), "mean_lower": statistics.mean(x["lower"] for x in rows),
               "mean_upper": statistics.mean(x["upper"] for x in rows),
               "mean_interval_width": statistics.mean(x["upper"] - x["lower"] for x in rows),
               "largest_instance_ratio_certificate": max(x["upper"] / x["lower"] for x in rows),
               "sample_maximum_upper": max(x["upper"] for x in rows),
               "planning_seconds": sum(x["planning_seconds"] for x in rows),
               "process_VmHWM": high_water}
    result = {"lower_source": str(args.input), "seed": source["seed"],
              "sampling": source["sampling"], "summary": summary, "targets": rows}
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
