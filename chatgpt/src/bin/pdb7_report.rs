//! Experimental report for the seven-card additive PDB heuristic.

use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use three_stack_shuffle::random::{Rng, SampleStats};
use three_stack_shuffle::search::{
    astar, astar_max_transport_partition_pdb, astar_max_transport_pdb, astar_partition_pdb,
    astar_pdb, transport_heuristic, PatternDatabases, PatternPartition, ReverseBfs,
};
use three_stack_shuffle::State;

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

fn current_rss_kib() -> Option<usize> {
    let statm = fs::read_to_string("/proc/self/statm").ok()?;
    let resident_pages = statm.split_whitespace().nth(1)?.parse::<usize>().ok()?;
    Some(resident_pages * 4096 / 1024)
}

fn print_stats(label: &str, stats: &SampleStats) {
    println!(
        "{label:<10} mean={:>8.3} min={:>4} max={:>4} stddev={:>8.3} stderr={:>8.3}",
        stats.mean,
        stats.minimum,
        stats.maximum,
        stats.standard_deviation(),
        stats.standard_error()
    );
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let n = value(&args, "--n", 52)? as usize;
    let samples = value(&args, "--samples", 1_000)? as usize;
    let seed = value(&args, "--seed", 0x5eed)?;
    let astar_n = value(&args, "--astar-n", 8)? as usize;
    let astar_samples = value(&args, "--astar-samples", 3)? as usize;

    println!("database construction");
    let rss_before = current_rss_kib();
    for size in 0..=7 {
        let began = Instant::now();
        let database = ReverseBfs::build(size);
        println!(
            "  size={size} states={} max_distance={} seconds={:.3}",
            database.len(),
            database.maximum_distance(),
            began.elapsed().as_secs_f64()
        );
    }
    let rss_after = current_rss_kib();
    if let (Some(before), Some(after)) = (rss_before, rss_after) {
        println!(
            "  rough_rss_change_kib={} (allocator may retain freed pages)",
            after.saturating_sub(before)
        );
    }

    let pdb = PatternDatabases::build(7);
    let mut rng = Rng::new(seed);
    let mut transport_stats = SampleStats::default();
    let mut value_pdb_stats = SampleStats::default();
    let mut value_max_stats = SampleStats::default();
    let mut order_pdb_stats = SampleStats::default();
    let mut order_max_stats = SampleStats::default();
    let mut value_max_improvement = SampleStats::default();
    let mut order_max_improvement = SampleStats::default();
    let mut value_pdb_gt_transport = 0_usize;
    let mut value_transport_gt_pdb = 0_usize;
    let mut value_equal = 0_usize;
    let mut order_pdb_gt_transport = 0_usize;
    let mut order_transport_gt_pdb = 0_usize;
    let mut order_equal = 0_usize;

    for _ in 0..samples {
        let deck = rng.permutation(n);
        let state = State::initial(&deck).map_err(|error| error.to_string())?;
        let order_partition = PatternPartition::from_order(&deck, 7);
        let transport = transport_heuristic(&state);
        let value_pdb = pdb.heuristic(&state, 7);
        let value_combined = transport.max(value_pdb);
        let order_pdb = pdb.heuristic_for_partition(&state, &order_partition);
        let order_combined = transport.max(order_pdb);
        transport_stats.add(transport);
        value_pdb_stats.add(value_pdb);
        value_max_stats.add(value_combined);
        order_pdb_stats.add(order_pdb);
        order_max_stats.add(order_combined);
        value_max_improvement.add(value_combined - transport);
        order_max_improvement.add(order_combined - transport);
        if value_pdb > transport {
            value_pdb_gt_transport += 1;
        } else if transport > value_pdb {
            value_transport_gt_pdb += 1;
        } else {
            value_equal += 1;
        }
        if order_pdb > transport {
            order_pdb_gt_transport += 1;
        } else if transport > order_pdb {
            order_transport_gt_pdb += 1;
        } else {
            order_equal += 1;
        }
    }

    println!();
    println!("random initial states: n={n} samples={samples} seed={seed}");
    print_stats("transport", &transport_stats);
    print_stats("value-pdb", &value_pdb_stats);
    print_stats("value-max", &value_max_stats);
    print_stats("order-pdb", &order_pdb_stats);
    print_stats("order-max", &order_max_stats);
    println!(
        "value fractions pdb>transport={:.3} transport>pdb={:.3} equal={:.3}",
        value_pdb_gt_transport as f64 / samples as f64,
        value_transport_gt_pdb as f64 / samples as f64,
        value_equal as f64 / samples as f64
    );
    println!(
        "order fractions pdb>transport={:.3} transport>pdb={:.3} equal={:.3}",
        order_pdb_gt_transport as f64 / samples as f64,
        order_transport_gt_pdb as f64 / samples as f64,
        order_equal as f64 / samples as f64
    );
    println!(
        "value improvement max_over_transport mean={:.3} max={}",
        value_max_improvement.mean, value_max_improvement.maximum
    );
    println!(
        "order improvement max_over_transport mean={:.3} max={}",
        order_max_improvement.mean, order_max_improvement.maximum
    );

    println!();
    println!("a-star comparison: n={astar_n} samples={astar_samples} seed={seed}");
    let exact = ReverseBfs::build(astar_n);
    let mut rng = Rng::new(seed);
    for sample in 0..astar_samples {
        let deck = rng.permutation(astar_n);
        let state = State::initial(&deck).map_err(|error| error.to_string())?;
        let order_partition = PatternPartition::from_order(&deck, 7);
        let distance = exact
            .distance(&state)
            .ok_or_else(|| "sample state missing from exact database".to_owned())?;
        let transport = astar(&state).map_err(|error| error.to_string())?;
        let value_pdb = astar_pdb(&state, &pdb, 7).map_err(|error| error.to_string())?;
        let value_combined =
            astar_max_transport_pdb(&state, &pdb, 7).map_err(|error| error.to_string())?;
        let order_pdb = astar_partition_pdb(&state, &pdb, &order_partition)
            .map_err(|error| error.to_string())?;
        let order_combined = astar_max_transport_partition_pdb(&state, &pdb, &order_partition)
            .map_err(|error| error.to_string())?;
        for (mode, result) in [
            ("transport", transport),
            ("value-pdb", value_pdb),
            ("value-max", value_combined),
            ("order-pdb", order_pdb),
            ("order-max", order_combined),
        ] {
            println!(
                "  sample={} mode={mode:<9} distance={} start_h={} generated={} expanded={} reopened={} stale={} max_open={} elapsed_ms={:.3}",
                sample + 1,
                distance,
                result.start_heuristic,
                result.stats.generated,
                result.stats.expanded,
                result.stats.reopened,
                result.stats.stale,
                result.stats.max_open,
                result.stats.elapsed.as_secs_f64() * 1000.0
            );
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("pdb7-report: {error}");
            ExitCode::FAILURE
        }
    }
}
