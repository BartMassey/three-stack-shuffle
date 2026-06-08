//! Scratch: test a candidate lower-bound potential Phi(sigma) against the exact
//! bounce optimum B_opt = (opt - 2m)/2. Admissibility requires Phi <= B_opt for
//! every deck. Candidate: the dyadic value-refinement sum -- sum over scales of
//! the static OCT of the value-coarsened departure order. See docs/NOTES.md
//! sec I.4a (the "multi-scale" proposal) and Part III (its refutation).

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{base_len, Card, State};
use splitmerge::search::ida_star;
use splitmerge::util::Rng;

/// Departure order sigma = above-base deck read top-to-bottom.
fn departure_order(deck: &[Card]) -> Vec<u16> {
    let k = base_len(deck);
    deck[k..].iter().rev().map(|&c| c as u16).collect()
}

/// Max cards coverable by two NON-INCREASING subsequences (equal allowed), by DP
/// over (index, topA, topB) with u16::MAX = empty pile. a2 = OCT complement.
fn a2_weak(seq: &[u16]) -> usize {
    use std::collections::HashMap;
    fn f(
        i: usize,
        ta: u16,
        tb: u16,
        seq: &[u16],
        memo: &mut HashMap<(usize, u16, u16), usize>,
    ) -> usize {
        if i == seq.len() {
            return 0;
        }
        if let Some(&v) = memo.get(&(i, ta, tb)) {
            return v;
        }
        let c = seq[i];
        let mut best = f(i + 1, ta, tb, seq, memo); // skip (delete) c
        if c <= ta {
            best = best.max(1 + f(i + 1, c, tb, seq, memo));
        }
        if c <= tb {
            best = best.max(1 + f(i + 1, ta, c, seq, memo));
        }
        memo.insert((i, ta, tb), best);
        best
    }
    let mut memo = HashMap::new();
    f(0, u16::MAX, u16::MAX, seq, &mut memo)
}

/// Static OCT at coarsening scale ell: value v -> (v-1) >> ell, then n - a2_weak.
fn oct_at_scale(sigma: &[u16], ell: u32) -> usize {
    let coarse: Vec<u16> = sigma.iter().map(|&v| (v - 1) >> ell).collect();
    sigma.len() - a2_weak(&coarse)
}

/// The dyadic refinement sum: sum_{ell >= 0} OCT^(ell).
fn phi_refine(sigma: &[u16]) -> usize {
    if sigma.is_empty() {
        return 0;
    }
    let maxv = *sigma.iter().max().unwrap();
    let levels = (16 - (maxv as u16).leading_zeros()) + 1; // enough to reach 1 block
    (0..levels).map(|ell| oct_at_scale(sigma, ell)).sum()
}

fn per_level(sigma: &[u16]) -> Vec<usize> {
    if sigma.is_empty() {
        return vec![];
    }
    let maxv = *sigma.iter().max().unwrap();
    let levels = (16 - (maxv as u16).leading_zeros()) + 1;
    (0..levels).map(|ell| oct_at_scale(sigma, ell)).collect()
}

fn b_opt(deck: &[Card]) -> Option<usize> {
    let st = State::from_deck(deck.to_vec());
    let m = deck.len() - base_len(deck);
    let (opt, _) = ida_star(&st, &h_joint, 8_000_000);
    opt.map(|o| (o as usize - 2 * m) / 2)
}

fn main() {
    // sanity: OCT^(0) must equal h_joint's bounce part for a clean deck.
    println!("=== sanity: OCT^(0) vs (h_joint - 2m)/2 on a few decks ===");
    let mut rng = Rng::new(1);
    for _ in 0..5 {
        let deck = rng.perm(9);
        let sigma = departure_order(&deck);
        let oct0 = oct_at_scale(&sigma, 0);
        let m = deck.len() - base_len(&deck);
        let hj = (h_joint(&State::from_deck(deck.clone())) as usize - 2 * m) / 2;
        println!("  deck {deck:?}  OCT^(0)={oct0}  (h_joint-2m)/2={hj}  {}", if oct0 == hj { "ok" } else { "MISMATCH" });
    }

    println!("\n=== reversed deck: per-level OCT and the sum vs B_opt = n-2 ===");
    for n in [6usize, 8, 10, 11] {
        let deck: Vec<Card> = (1..=n as Card).rev().collect();
        let sigma = departure_order(&deck);
        let levels = per_level(&sigma);
        let phi = phi_refine(&sigma);
        println!("  n={n:2}: levels={levels:?} sum={phi}  B_opt={}  {}",
            n - 2, if phi <= n - 2 { "admissible" } else { "OVERSHOOTS" });
    }

    println!("\n=== Phi_refine vs B_opt on random decks (admissibility needs max<=0) ===");
    println!("  n | mean B_opt | mean Phi | max(Phi-B_opt) | #overshoot/num | Phi/(n log2 n)");
    for n in 6..=11usize {
        let mut rng = Rng::new(100 + n as u64);
        let num = 40;
        let (mut sb, mut sp, mut maxgap, mut over, mut cnt) = (0.0, 0.0, i64::MIN, 0, 0);
        for _ in 0..num {
            let deck = rng.perm(n);
            let sigma = departure_order(&deck);
            let phi = phi_refine(&sigma) as i64;
            let Some(b) = b_opt(&deck) else { continue };
            let b = b as i64;
            sb += b as f64;
            sp += phi as f64;
            let gap = phi - b;
            if gap > maxgap { maxgap = gap; }
            if gap > 0 { over += 1; }
            cnt += 1;
        }
        let c = cnt as f64;
        let nln = n as f64 * (n as f64).log2();
        println!("  {n:2} |   {:6.2}   |  {:6.2}  |      {:+3}       |     {over:2}/{cnt}      |   {:.3}",
            sb / c, sp / c, maxgap, (sp / c) / nln);
    }
}
