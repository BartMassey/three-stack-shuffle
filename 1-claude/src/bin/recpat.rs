//! Experiment (c): RECURSIVE patience — idealized cost model.
//!
//! Single-pass 2-pile patience sorts only `LDS<=2` decks (psort). Recurse: split the
//! sequence into two subsequences each with ~half the LDS (Dilworth cover into LDS
//! increasing chains, 2-color the chains), sort each, merge. Depth ~log2(LDS), each
//! level costs 2*(cards) -> total ~ 2n*log2(LDS), the "log r -> log LDS" lever.
//!
//! This is IDEALIZED (like revsort's free-reversal): it charges 2*len per level and
//! assumes the LDS-halving split is realizable in `len` moves. Real routing of cards to
//! non-adjacent positions may cost more — so this is a LOWER bound / best case. If it
//! does NOT beat Hu-Tucker on random, recursion can't help; if it does, realizability is
//! the open question. Compared to Hu-Tucker (committed, realizable) and natural merge.

use splitmerge::machine::Card;
use splitmerge::sorters::{ascending_runs, hutucker_cost, natural_cost};
use splitmerge::util::Rng;

fn is_ascending(s: &[Card]) -> bool {
    s.windows(2).all(|w| w[0] < w[1])
}

/// Longest strictly decreasing subsequence length.
fn lds(s: &[Card]) -> usize {
    let n = s.len();
    let mut dp = vec![1usize; n];
    let mut best = 0;
    for i in 0..n {
        for j in 0..i {
            if s[j] > s[i] && dp[j] + 1 > dp[i] {
                dp[i] = dp[j] + 1;
            }
        }
        best = best.max(dp[i]);
    }
    best
}

/// Dilworth cover into increasing chains (greedy patience); returns chain index per elt.
/// #chains == LDS. Each chain is an increasing subsequence.
fn cover_chains(s: &[Card]) -> (Vec<usize>, usize) {
    let mut tops: Vec<Card> = Vec::new(); // top (max) of each increasing chain
    let mut idx = vec![0usize; s.len()];
    for (p, &c) in s.iter().enumerate() {
        // leftmost chain whose top < c (extend it, staying increasing)
        let mut placed = None;
        for (ci, t) in tops.iter_mut().enumerate() {
            if *t < c {
                *t = c;
                placed = Some(ci);
                break;
            }
        }
        idx[p] = placed.unwrap_or_else(|| {
            tops.push(c);
            tops.len() - 1
        });
    }
    (idx, tops.len())
}

/// Split into two subsequences (original order) by 2-coloring the increasing chains;
/// each part has LDS <= ceil(chains/2) < LDS for LDS>=3.
fn split_halving(s: &[Card]) -> (Vec<Card>, Vec<Card>) {
    let (idx, _) = cover_chains(s);
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for (p, &c) in s.iter().enumerate() {
        if idx[p] % 2 == 0 {
            a.push(c);
        } else {
            b.push(c);
        }
    }
    (a, b)
}

/// Idealized recursive-patience move count.
fn rec_cost(s: &[Card]) -> usize {
    if s.len() <= 1 || is_ascending(s) {
        return 0; // already sorted: free
    }
    if lds(s) <= 2 {
        return 2 * s.len(); // one 2-pile patience pass + merge
    }
    let (a, b) = split_halving(s);
    // progress guard (must hold for LDS>=3 with >=2 chains)
    debug_assert!(a.len() < s.len() && b.len() < s.len());
    2 * s.len() + rec_cost(&a) + rec_cost(&b)
}

/// max recursion depth (for intuition).
fn rec_depth(s: &[Card]) -> usize {
    if s.len() <= 1 || is_ascending(s) || lds(s) <= 2 {
        return 1;
    }
    let (a, b) = split_halving(s);
    1 + rec_depth(&a).max(rec_depth(&b))
}

fn report(label: &str, deck: &[Card]) {
    println!(
        "{label:<22} rec_patience(ideal)={:>4}  hutucker={:>4}  natural={:>4}  | lds={} depth={} asc_runs={}",
        rec_cost(deck), hutucker_cost(deck), natural_cost(deck), lds(deck), rec_depth(deck), ascending_runs(deck).len()
    );
}

fn main() {
    let n = 52usize;

    // sanity: cover-chain count must equal LDS.
    {
        let mut rng = Rng::new(3);
        let mut ok = true;
        for _ in 0..500 {
            let p = rng.perm(n);
            if cover_chains(&p).1 != lds(&p) {
                ok = false;
                break;
            }
        }
        println!("sanity: #cover-chains == LDS over 500 random n=52: {ok}\n");
    }

    let reversed: Vec<Card> = (1..=n as Card).rev().collect();
    report("reversed", &reversed);
    let k = n / 2;
    let mut inter: Vec<Card> = Vec::new();
    for t in 0..k {
        inter.push(1 + t as Card);
        inter.push(1 + (k + t) as Card);
    }
    report("interleave(obvious)", &inter);

    // random average
    println!();
    let trials = 5000usize;
    let mut rng = Rng::new(11);
    let (mut sr, mut sh, mut sn, mut sdepth, mut slds) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut rec_wins = 0usize;
    for _ in 0..trials {
        let p = rng.perm(n);
        let r = rec_cost(&p) as u64;
        let h = hutucker_cost(&p) as u64;
        sr += r;
        sh += h;
        sn += natural_cost(&p) as u64;
        sdepth += rec_depth(&p) as u64;
        slds += lds(&p) as u64;
        if r < h {
            rec_wins += 1;
        }
    }
    let m = |x: u64| x as f64 / trials as f64;
    println!("random n=52 ({trials} decks):");
    println!(
        "  rec_patience(ideal)={:.1}  hutucker={:.1}  natural={:.1}",
        m(sr), m(sh), m(sn)
    );
    println!(
        "  rec beats hutucker on {rec_wins}/{trials} decks; mean improvement {:.1} ({:.1}%)",
        m(sh) - m(sr),
        100.0 * (m(sh) - m(sr)) / m(sh)
    );
    println!(
        "  mean LDS={:.1}  mean depth={:.2}  (theory: 2n*log2(LDS) = {:.0})",
        m(slds),
        m(sdepth),
        2.0 * n as f64 * (m(slds)).log2()
    );

    // scaling with n (does the % improvement grow, as log r - log LDS should?)
    println!("\nrec(ideal) vs hutucker mean, by n:");
    for nn in [16usize, 24, 32, 40, 52, 80, 120] {
        let mut rr = Rng::new(500 + nn as u64);
        let (mut a, mut b, t) = (0u64, 0u64, 1500usize);
        for _ in 0..t {
            let p = rr.perm(nn);
            a += rec_cost(&p) as u64;
            b += hutucker_cost(&p) as u64;
        }
        println!(
            "  n={nn:>3}: rec={:.1}  hutucker={:.1}  improvement {:.1}%",
            a as f64 / t as f64,
            b as f64 / t as f64,
            100.0 * (b - a) as f64 / b as f64
        );
    }
}
