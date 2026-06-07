//! Experiment runner (the ports of `python/experiments/*`, plus the opt-vs-merge
//! frontier measurement). Subcommands:
//!
//!   sm heuristics [n] [num]      h_best vs h_joint node expansions (IDA*)
//!   sm sorters    [n] [num]      merge-sorter averages and worst cases
//!   sm conjecture [n] [num]      does any deck beat 4(n-1)?
//!   sm planner    [n] [budget] [num]   first / restart / ILS at size n
//!   sm cascade    [max_n] [num]  cascade charge vs exact opt (tightness)
//!   sm frontier   [budget] [num] opt vs merge vs ILS across n (the headline)

use std::time::Instant;

use splitmerge::heuristics::{h_best, h_joint};
use splitmerge::machine::State;
use splitmerge::planner::{cascade_charge, completion, iterated_local_search, local_search, rollout_cost, rollout_merge};
use splitmerge::search::ida_star;
use splitmerge::sorters::{hutucker_cost, natural_cost, topdown_cost};
use splitmerge::util::Rng;

fn arg(i: usize, default: &str) -> String {
    std::env::args().nth(i).unwrap_or_else(|| default.to_string())
}
fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}
fn log2(n: usize) -> f64 {
    (n as f64).log2()
}

fn heuristics(n: usize, num: usize) {
    let mut rng = Rng::new(11);
    let decks: Vec<State> = (0..num).map(|_| State::from_deck(rng.perm(n))).collect();
    for (name, h) in [("h_best", h_best as fn(&State) -> u32), ("h_joint", h_joint)] {
        let t = Instant::now();
        let mut nodes = Vec::new();
        let mut costs = Vec::new();
        for st in &decks {
            let (c, nd) = ida_star(st, &h, 2_000_000);
            if let Some(c) = c {
                costs.push(c);
                nodes.push(nd);
            }
        }
        nodes.sort_unstable();
        println!(
            "  {:8} median={:7} max={:8} total={:9} time={:5.2}s",
            name,
            nodes[nodes.len() / 2],
            nodes.iter().max().unwrap(),
            nodes.iter().sum::<u64>(),
            t.elapsed().as_secs_f64()
        );
    }
}

fn sorters(n: usize, num: usize) {
    let mut rng = Rng::new(2024);
    let (mut nat, mut td, mut ht) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..num {
        let p = rng.perm(n);
        nat.push(natural_cost(&p) as f64);
        td.push(topdown_cost(&p) as f64);
        ht.push(hutucker_cost(&p) as f64);
    }
    let rev: Vec<u8> = (1..=n as u8).rev().collect();
    println!("  n={n}, {num} random decks");
    for (name, v, rc) in [
        ("natural", &nat, natural_cost(&rev)),
        ("topdown", &td, topdown_cost(&rev)),
        ("hutucker", &ht, hutucker_cost(&rev)),
    ] {
        println!(
            "    {:8} avg={:6.1} min={:.0} max={:.0} worst(reversed)={}",
            name,
            mean(v),
            v.iter().cloned().fold(f64::INFINITY, f64::min),
            v.iter().cloned().fold(0.0, f64::max),
            rc
        );
    }
}

fn conjecture(n: usize, num: usize) {
    let target = 4 * (n as u32 - 1);
    let mut rng = Rng::new(5);
    let (mut best, mut over) = (0u32, 0);
    for _ in 0..num {
        if let (Some(c), _) = ida_star(&State::from_deck(rng.perm(n)), &h_joint, 300_000) {
            best = best.max(c);
            if c > target {
                over += 1;
            }
        }
    }
    println!("  n={n}: target 4(n-1)={target}  random max={best}  over={over}/{num}");
}

fn planner(n: usize, budget: f64, num: usize) {
    println!("n={n}, anytime budget {budget:.0}s");
    let report = |name: &str, st: &State| {
        let t = Instant::now();
        let (_, first) = completion(st);
        let ft = t.elapsed().as_secs_f64() * 1e3;
        let settle = rollout_cost(st);
        let merge = rollout_merge(st).len();
        let lb = h_joint(st);
        let ls = local_search(st, budget, 1, None).best_cost;
        let ils = iterated_local_search(st, budget, 1, 6);
        println!(
            "  {name:10} first={first:5} ({ft:6.1}ms) [settle={settle:5} merge={merge:5}] h_joint(LB)={lb:4} first/LB={:.2} restart={ls:5} ILS={:5} (x{})",
            first as f64 / lb as f64,
            ils.best_cost,
            ils.iters
        );
    };
    report("reversed", &State::reversed_deck(n));
    let mut rng = Rng::new(7);
    for i in 0..num {
        report(&format!("random#{i}"), &State::from_deck(rng.perm(n)));
    }
}

fn cascade(max_n: usize, num: usize) {
    let mut rng = Rng::new(4);
    println!("  n :  h_joint_gap(opt-LB)   cascade_overshoot(cas-opt)   max   overshoots");
    for n in 6..=max_n {
        let (mut hj, mut og, mut over) = (Vec::new(), Vec::new(), 0);
        for _ in 0..num {
            let st = State::from_deck(rng.perm(n));
            let opt = ida_star(&st, &h_joint, 3_000_000).0.unwrap() as i64;
            hj.push((opt - h_joint(&st) as i64) as f64);
            let cas = cascade_charge(&st) as i64;
            og.push((cas - opt) as f64);
            over += (cas > opt) as i32;
        }
        println!(
            "  {n:2}:      {:5.2}                {:6.2}              {:3.0}    {over}/{num}",
            mean(&hj),
            mean(&og),
            og.iter().cloned().fold(0.0, f64::max)
        );
    }
}

fn frontier(budget: f64, num: usize) {
    println!(" n  opt   opt/(n log2 n)  merge  merge/opt  ILS  ILS/merge  (opt only n<=13)");
    for n in [10usize, 11, 12, 13, 16, 19, 22, 26, 30, 40, 52] {
        let mut rng = Rng::new(200 + n as u64);
        let (mut opts, mut merges, mut ilss) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..num {
            let st = State::from_deck(rng.perm(n));
            let merge = hutucker_cost(&st.d) as f64;
            merges.push(merge);
            ilss.push(iterated_local_search(&st, budget, 1, 6).best_cost as f64);
            if n <= 13 {
                if let (Some(o), _) = ida_star(&st, &h_joint, 5_000_000) {
                    opts.push(o as f64);
                }
            }
        }
        let merge = mean(&merges);
        let ils = mean(&ilss);
        if !opts.is_empty() {
            let opt = mean(&opts);
            println!(
                " {n:2} {opt:5.1}     {:.3}      {merge:5.0}    {:.2}    {ils:4.0}   {:.3}",
                opt / (n as f64 * log2(n)),
                merge / opt,
                ils / merge
            );
        } else {
            println!(" {n:2}   -          -        {merge:5.0}     -      {ils:4.0}   {:.3}", ils / merge);
        }
    }
}

fn main() {
    let cmd = arg(1, "frontier");
    match cmd.as_str() {
        "heuristics" => heuristics(arg(2, "10").parse().unwrap(), arg(3, "80").parse().unwrap()),
        "sorters" => sorters(arg(2, "52").parse().unwrap(), arg(3, "2000").parse().unwrap()),
        "conjecture" => conjecture(arg(2, "10").parse().unwrap(), arg(3, "150").parse().unwrap()),
        "planner" => planner(
            arg(2, "52").parse().unwrap(),
            arg(3, "6").parse().unwrap(),
            arg(4, "5").parse().unwrap(),
        ),
        "cascade" => cascade(arg(2, "11").parse().unwrap(), arg(3, "40").parse().unwrap()),
        "frontier" => frontier(arg(2, "5").parse().unwrap(), arg(3, "8").parse().unwrap()),
        other => eprintln!("unknown subcommand: {other}"),
    }
}
