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
                println!(
                    "  {{\"algorithm\":\"{}\",\"experimental\":{},\"n\":{n},\"samples\":{},\"seed\":{seed},\"mean\":{:.9},\"standard_deviation\":{:.9},\"standard_error\":{:.9},\"minimum\":{},\"maximum\":{},\"transport_lower_bound_mean\":{:.9},\"transport_lower_bound_standard_deviation\":{:.9},\"mean_gap_vs_lower_bound\":{:.9},\"mean_ratio_vs_lower_bound\":{:.9},\"elapsed_seconds\":{:.6}}}{}",
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
