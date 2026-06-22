//! Command-line random correctness campaign.

use std::process::ExitCode;

use three_stack_shuffle::algorithms::Algorithm;
use three_stack_shuffle::random::random_test;

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

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().collect();
    let run = || -> Result<(), String> {
        let n = value(&args, "--n", 52)? as usize;
        let samples = value(&args, "--samples", 1_000)? as usize;
        let seed = value(&args, "--seed", 0x5eed)?;
        let report =
            random_test(n, samples, seed, &Algorithm::ALL).map_err(|error| error.to_string())?;
        if args.iter().any(|arg| arg == "--json") {
            println!(
                "{{\"n\":{n},\"samples\":{},\"seed\":{seed},\"plans_checked\":{},\"elapsed_seconds\":{:.6}}}",
                report.samples,
                report.plans_checked,
                report.elapsed.as_secs_f64()
            );
        } else {
            println!(
                "random test passed: n={n}, samples={}, seed={seed}, plans={}, elapsed={:.3}s",
                report.samples,
                report.plans_checked,
                report.elapsed.as_secs_f64()
            );
        }
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("random-test: {error}");
            ExitCode::FAILURE
        }
    }
}
