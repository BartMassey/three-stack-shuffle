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
                println!(
                    "  {{\"algorithm\":\"{}\",\"experimental\":{},\"n\":{n},\"samples\":{},\"seed\":{seed},\"mean\":{:.9},\"standard_deviation\":{:.9},\"standard_error\":{:.9},\"minimum\":{},\"maximum\":{},\"elapsed_seconds\":{:.6}}}{}",
                    report.algorithm.name(),
                    report.algorithm.is_experimental(),
                    report.moves.count,
                    report.moves.mean,
                    report.moves.standard_deviation(),
                    report.moves.standard_error(),
                    report.moves.minimum,
                    report.moves.maximum,
                    report.elapsed.as_secs_f64(),
                    if index + 1 == reports.len() { "" } else { "," }
                );
            }
            println!("]");
        } else {
            println!("n={n}, samples={samples}, seed={seed}");
            println!("algorithm                             mean      stddev      stderr    min    max  seconds");
            for report in reports {
                println!(
                    "{:<35} {:>9.3} {:>11.3} {:>11.3} {:>6} {:>6} {:>8.3}",
                    report.algorithm.name(),
                    report.moves.mean,
                    report.moves.standard_deviation(),
                    report.moves.standard_error(),
                    report.moves.minimum,
                    report.moves.maximum,
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
