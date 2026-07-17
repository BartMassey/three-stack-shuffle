//! Command-line random move-count and timing benchmark.

use std::process::ExitCode;

use three_stack_shuffle::algorithms::Algorithm;
use three_stack_shuffle::random::random_benchmark;

fn value(args: &[String], flag: &str, default: u64) -> Result<u64, String> {
    match args.iter().position(|arg| arg == flag) {
        Some(index) => args
            .get(index + 1)
            .ok_or_else(|| format!("missing value after {flag}"))?
            .parse()
            .map_err(|_| format!("invalid value after {flag}")),
        None => Ok(default),
    }
}

fn selected_algorithms(args: &[String]) -> Result<Vec<Algorithm>, String> {
    let Some(index) = args.iter().position(|arg| arg == "--algorithm") else {
        return Ok(Algorithm::ALL.to_vec());
    };
    let name = args
        .get(index + 1)
        .ok_or_else(|| "missing value after --algorithm".to_owned())?;
    Algorithm::from_name(name)
        .map(|algorithm| vec![algorithm])
        .ok_or_else(|| format!("unknown algorithm {name}"))
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();
    let run = || -> Result<(), String> {
        let n = value(&args, "--n", 52)? as usize;
        let samples = value(&args, "--samples", 20_000)? as usize;
        let seed = value(&args, "--seed", 0x5eed)?;
        let algorithms = selected_algorithms(&args)?;
        let reports =
            random_benchmark(n, samples, seed, &algorithms).map_err(|error| error.to_string())?;
        if args.iter().any(|arg| arg == "--json") {
            println!("[");
            for (index, report) in reports.iter().enumerate() {
                let mean_gap = report.moves.mean - report.lower_bounds.mean;
                let mean_ratio = report.moves.mean / report.lower_bounds.mean;
                let planning_seconds = report.incremental_rhl.planning_nanos as f64 / 1e9;
                let planning_per_target_micros = if report.incremental_rhl.planning_targets == 0 {
                    0.0
                } else {
                    report.incremental_rhl.planning_nanos as f64
                        / report.incremental_rhl.planning_targets as f64
                        / 1e3
                };
                let planning_per_bucket_micros = if report.incremental_rhl.planning_buckets == 0 {
                    0.0
                } else {
                    report.incremental_rhl.planning_nanos as f64
                        / report.incremental_rhl.planning_buckets as f64
                        / 1e3
                };
                let depth_planning_seconds = report.depth_limited_rhl.planning_nanos as f64 / 1e9;
                let depth_planning_per_decision_micros =
                    if report.depth_limited_rhl.planning_decisions == 0 {
                        0.0
                    } else {
                        report.depth_limited_rhl.planning_nanos as f64
                            / report.depth_limited_rhl.planning_decisions as f64
                            / 1e3
                    };
                println!(
                    "  {{\"algorithm\":\"{}\",\"experimental\":{},\"n\":{n},\"samples\":{},\"seed\":{seed},\"mean\":{:.9},\"standard_deviation\":{:.9},\"standard_error\":{:.9},\"minimum\":{},\"maximum\":{},\"transport_lower_bound_mean\":{:.9},\"transport_lower_bound_standard_deviation\":{:.9},\"mean_gap_vs_lower_bound\":{:.9},\"mean_ratio_vs_lower_bound\":{:.9},\"elapsed_seconds\":{:.6},\"masks_visited\":{},\"distinct_algebraic_successors\":{},\"distinct_normalized_successors\":{},\"base_cache_hits\":{},\"base_cache_misses\":{},\"base_states_stored\":{},\"forced_targets_removed\":{},\"estimated_peak_memory_bytes\":{},\"planning_seconds\":{planning_seconds:.6},\"planning_per_target_micros\":{planning_per_target_micros:.3},\"planning_per_bucket_micros\":{planning_per_bucket_micros:.3},\"depth\":{},\"binary_nodes_expanded\":{},\"frontier_evaluations\":{},\"greedy_cache_hits\":{},\"greedy_cache_misses\":{},\"suffix_cache_hits\":{},\"suffix_cache_misses\":{},\"suffix_states_stored\":{},\"suffix_forced_targets_removed\":{},\"depth_cache_hits\":{},\"depth_cache_misses\":{},\"nodes_retained_after_rerooting\":{},\"new_nodes_added\":{},\"depth_peak_memory_bytes\":{},\"depth_planning_seconds\":{depth_planning_seconds:.6},\"depth_planning_per_decision_micros\":{depth_planning_per_decision_micros:.3},\"target_block_states\":{},\"target_block_transitions\":{},\"target_block_max_candidates\":{},\"target_block_cache_hits\":{},\"target_block_forced_targets\":{}}}{}",
                    report.algorithm.name(),
                    report.algorithm.is_experimental(),
                    report.moves.count,
                    report.moves.mean,
                    report.moves.standard_deviation(),
                    report.moves.standard_error(),
                    report.moves.minimum,
                    report.moves.maximum,
                    report.lower_bounds.mean,
                    report.lower_bounds.standard_deviation(),
                    mean_gap,
                    mean_ratio,
                    report.elapsed.as_secs_f64(),
                    report.incremental_rhl.masks_visited,
                    report.incremental_rhl.distinct_algebraic_successors,
                    report.incremental_rhl.distinct_normalized_successors,
                    report.incremental_rhl.base_cache_hits,
                    report.incremental_rhl.base_cache_misses,
                    report.incremental_rhl.base_states_stored,
                    report.incremental_rhl.forced_targets_removed,
                    report.incremental_rhl.estimated_peak_memory_bytes,
                    report.depth_limited_rhl.depth,
                    report.depth_limited_rhl.binary_nodes_expanded,
                    report.depth_limited_rhl.frontier_evaluations,
                    report.depth_limited_rhl.greedy_cache_hits,
                    report.depth_limited_rhl.greedy_cache_misses,
                    report.depth_limited_rhl.suffix_cache_hits,
                    report.depth_limited_rhl.suffix_cache_misses,
                    report.depth_limited_rhl.suffix_states_stored,
                    report.depth_limited_rhl.suffix_forced_targets_removed,
                    report.depth_limited_rhl.depth_cache_hits,
                    report.depth_limited_rhl.depth_cache_misses,
                    report.depth_limited_rhl.nodes_retained_after_rerooting,
                    report.depth_limited_rhl.new_nodes_added,
                    report.depth_limited_rhl.estimated_peak_memory_bytes,
                    report.target_block_dp.states,
                    report.target_block_dp.transitions,
                    report.target_block_dp.max_candidates,
                    report.target_block_dp.cache_hits,
                    report.target_block_dp.forced_targets,
                    if index + 1 == reports.len() { "" } else { "," }
                );
            }
            println!("]");
        } else {
            println!("n={n}, samples={samples}, seed={seed}");
            println!(
                "algorithm                             mean      stddev      stderr    min    max   lb mean       gap   ratio  seconds"
            );
            for report in reports {
                let mean_gap = report.moves.mean - report.lower_bounds.mean;
                let mean_ratio = report.moves.mean / report.lower_bounds.mean;
                println!(
                    "{:<35} {:>9.3} {:>11.3} {:>11.3} {:>6} {:>6} {:>9.3} {:>9.3} {:>7.3} {:>8.3}",
                    report.algorithm.name(),
                    report.moves.mean,
                    report.moves.standard_deviation(),
                    report.moves.standard_error(),
                    report.moves.minimum,
                    report.moves.maximum,
                    report.lower_bounds.mean,
                    mean_gap,
                    mean_ratio,
                    report.elapsed.as_secs_f64()
                );
                if report.incremental_rhl.planning_buckets > 0 {
                    let planning_per_target = report.incremental_rhl.planning_nanos as f64
                        / report.incremental_rhl.planning_targets as f64
                        / 1e3;
                    let planning_per_bucket = report.incremental_rhl.planning_nanos as f64
                        / report.incremental_rhl.planning_buckets as f64
                        / 1e3;
                    println!(
                        "  incremental-rhl: masks={} algebraic={} normalized={} cache_hits={} cache_misses={} stored={} forced={} peak_memory_bytes={} planning_us/target={planning_per_target:.3} planning_us/bucket={planning_per_bucket:.3}",
                        report.incremental_rhl.masks_visited,
                        report.incremental_rhl.distinct_algebraic_successors,
                        report.incremental_rhl.distinct_normalized_successors,
                        report.incremental_rhl.base_cache_hits,
                        report.incremental_rhl.base_cache_misses,
                        report.incremental_rhl.base_states_stored,
                        report.incremental_rhl.forced_targets_removed,
                        report.incremental_rhl.estimated_peak_memory_bytes,
                    );
                }
                if report.depth_limited_rhl.planning_buckets > 0 {
                    let planning_per_decision = if report.depth_limited_rhl.planning_decisions == 0
                    {
                        0.0
                    } else {
                        report.depth_limited_rhl.planning_nanos as f64
                            / report.depth_limited_rhl.planning_decisions as f64
                            / 1e3
                    };
                    println!(
                        "  depth-limited-rhl: depth={} nodes={} frontier={} greedy_hits={} greedy_misses={} suffix_hits={} suffix_misses={} suffix_stored={} suffix_forced={} depth_hits={} depth_misses={} retained={} new_nodes={} peak_memory_bytes={} planning_us/decision={planning_per_decision:.3}",
                        report.depth_limited_rhl.depth,
                        report.depth_limited_rhl.binary_nodes_expanded,
                        report.depth_limited_rhl.frontier_evaluations,
                        report.depth_limited_rhl.greedy_cache_hits,
                        report.depth_limited_rhl.greedy_cache_misses,
                        report.depth_limited_rhl.suffix_cache_hits,
                        report.depth_limited_rhl.suffix_cache_misses,
                        report.depth_limited_rhl.suffix_states_stored,
                        report.depth_limited_rhl.suffix_forced_targets_removed,
                        report.depth_limited_rhl.depth_cache_hits,
                        report.depth_limited_rhl.depth_cache_misses,
                        report.depth_limited_rhl.nodes_retained_after_rerooting,
                        report.depth_limited_rhl.new_nodes_added,
                        report.depth_limited_rhl.estimated_peak_memory_bytes,
                    );
                }
                if report.target_block_dp.states > 0 {
                    println!(
                        "  target-block-rollout: suffix_states={} candidates={} max_candidates={} cache_hits={} forced_targets={} mean_suffix_states/sample={:.3} mean_candidates/sample={:.3}",
                        report.target_block_dp.states,
                        report.target_block_dp.transitions,
                        report.target_block_dp.max_candidates,
                        report.target_block_dp.cache_hits,
                        report.target_block_dp.forced_targets,
                        report.target_block_dp.states as f64 / report.moves.count as f64,
                        report.target_block_dp.transitions as f64 / report.moves.count as f64,
                    );
                }
            }
        }
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("random-bench: {error}");
            ExitCode::FAILURE
        }
    }
}
