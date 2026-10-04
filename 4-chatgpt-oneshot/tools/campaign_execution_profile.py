import argparse
import cProfile
import json
from pathlib import Path
import pstats
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from parked_leaf import parked_recommended
from solver import recommended


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    development = json.loads((ROOT / "results/campaign-execution-development.json").read_text())
    targets = [row["target"] for row in development["targets"]]
    initial = list(range(52))
    results = {}
    for name, algorithm in (("recommended", recommended),
                            ("parked_recommended", parked_recommended)):
        cold_code = (
            "import json,resource,time;"
            f"from {'solver' if name == 'recommended' else 'parked_leaf'} import {name};"
            f"target={targets[0]!r};start=time.perf_counter();"
            f"word={name}(list(range(52)),target);"
            "elapsed=time.perf_counter()-start;"
            "print(json.dumps({'seconds':elapsed,'moves':len(word),"
            "'rss_kib':resource.getrusage(resource.RUSAGE_SELF).ru_maxrss}))"
        )
        cold = json.loads(subprocess.check_output([sys.executable, "-c", cold_code],
                                                  cwd=ROOT, text=True))
        algorithm(initial, targets[0])
        profiler = cProfile.Profile()
        profiler.enable()
        for target in targets:
            algorithm(initial, target)
        profiler.disable()
        stats = pstats.Stats(profiler)
        rows = []
        for (filename, line, function), (primitive, calls, total, cumulative, callers) in stats.stats.items():
            rows.append({"file": filename, "line": line, "function": function,
                         "primitive_calls": primitive, "calls": calls,
                         "self_seconds": total, "cumulative_seconds": cumulative})
        results[name] = {"cold_planning": cold, "profiled_target_count": len(targets),
                         "profiled_total_seconds": stats.total_tt,
                         "top_self": sorted(rows, key=lambda row: row["self_seconds"], reverse=True)[:20],
                         "top_cumulative": sorted(rows, key=lambda row: row["cumulative_seconds"], reverse=True)[:20]}
    args.output.write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps({name: {"cold": result["cold_planning"],
                             "top_self": result["top_self"][:5]}
                      for name, result in results.items()}, indent=2))


if __name__ == "__main__":
    main()
