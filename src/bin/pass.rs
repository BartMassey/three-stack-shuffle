//! Experiment (d): is there a REALIZABLE multi-pass that halves LDS?
//!
//! A pass is fully realizable: distribute all of D onto A,B (n moves), then recombine
//! all back to D (n moves) = exactly 2n moves. Iterate until sorted. If each pass roughly
//! halves LDS, then ~log2(LDS) passes sort the deck (vs natural merge's log2(r) passes) —
//! turning recpat's idealized 24% into a real sorter. We measure the LDS trajectory and
//! passes-to-sort for a patience distribute + greedy-merge recombine, replay-checking that
//! every pass is a valid machine permutation and the final deck is sorted.

use splitmerge::machine::{Card, Move, State};
use splitmerge::sorters::{ascending_runs, hutucker_cost, natural_cost};
use splitmerge::util::Rng;

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

struct Emitter {
    d: Vec<Card>,
    a: Vec<Card>,
    b: Vec<Card>,
    moves: Vec<Move>,
}
impl Emitter {
    fn new(deck: &[Card]) -> Self {
        Emitter { d: deck.to_vec(), a: Vec::new(), b: Vec::new(), moves: Vec::new() }
    }
    fn sa(&mut self) { let c = self.d.pop().unwrap(); self.a.push(c); self.moves.push(Move::SA); }
    fn sb(&mut self) { let c = self.d.pop().unwrap(); self.b.push(c); self.moves.push(Move::SB); }
    fn ma(&mut self) { let c = self.a.pop().unwrap(); self.d.push(c); self.moves.push(Move::MA); }
    fn mb(&mut self) { let c = self.b.pop().unwrap(); self.d.push(c); self.moves.push(Move::MB); }
}

/// One pass: patience best-fit distribute (bury on overflow), then greedy-merge recombine.
/// `bury_larger`: on overflow (c > both tops) place on the larger-top pile (else smaller).
fn one_pass(m: &mut Emitter, bury_larger: bool) {
    // distribute
    while let Some(&c) = m.d.last() {
        let a_fits = m.a.last().map_or(false, |&t| t > c);
        let b_fits = m.b.last().map_or(false, |&t| t > c);
        match (a_fits, b_fits) {
            (true, true) => if m.a.last() <= m.b.last() { m.sa() } else { m.sb() },
            (true, false) => m.sa(),
            (false, true) => m.sb(),
            (false, false) => {
                if m.a.is_empty() { m.sa() }
                else if m.b.is_empty() { m.sb() }
                else {
                    // overflow: both nonempty, c > both tops
                    let a_big = m.a.last() >= m.b.last();
                    if a_big == bury_larger { m.sa() } else { m.sb() }
                }
            }
        }
    }
    // recombine: greedy merge (smaller top first)
    while !m.a.is_empty() || !m.b.is_empty() {
        match (m.a.last(), m.b.last()) {
            (Some(x), Some(y)) => if x < y { m.ma() } else { m.mb() },
            (Some(_), None) => m.ma(),
            (None, Some(_)) => m.mb(),
            (None, None) => break,
        }
    }
}

/// Iterate passes until sorted or `cap`. Returns (passes, sorted?, lds_trajectory).
fn run(deck: &[Card], bury_larger: bool, cap: usize) -> (usize, bool, Vec<usize>, Vec<Move>) {
    let mut m = Emitter::new(deck);
    let mut traj = vec![lds(&m.d)];
    let mut passes = 0;
    while ascending_runs(&m.d).len() > 1 && passes < cap {
        one_pass(&mut m, bury_larger);
        passes += 1;
        traj.push(lds(&m.d));
    }
    let sorted = ascending_runs(&m.d).len() <= 1;
    (passes, sorted, traj, m.moves)
}

fn check(deck: &[Card], moves: &[Move]) -> bool {
    State::from_deck(deck.to_vec()).applied(moves) == State::goal(deck.len())
}

/// Dilworth increasing-chain index per element (count == LDS).
fn cover_chains(s: &[Card]) -> Vec<usize> {
    let mut tops: Vec<Card> = Vec::new();
    let mut idx = vec![0usize; s.len()];
    for (p, &c) in s.iter().enumerate() {
        let mut placed = None;
        for (ci, t) in tops.iter_mut().enumerate() {
            if *t < c { *t = c; placed = Some(ci); break; }
        }
        idx[p] = placed.unwrap_or_else(|| { tops.push(c); tops.len() - 1 });
    }
    idx
}

/// One pass distributing by chain-PARITY (the recpat split: even chains -> A, odd -> B),
/// then greedy-merge recombine. The split halves LDS; does the realizable recombine keep
/// it halved on D? `m.d` is bottom..top, so departure order (pop) is reversed.
fn one_pass_chainparity(m: &mut Emitter) {
    // chains over the DEPARTURE order (= d reversed)
    let dep: Vec<Card> = m.d.iter().rev().cloned().collect();
    let chains = cover_chains(&dep);
    // map back: position in d (bottom..top) is dep index (n-1-pos)
    let n = m.d.len();
    let mut to_a = vec![false; n];
    for (depi, &ch) in chains.iter().enumerate() {
        to_a[n - 1 - depi] = ch % 2 == 0;
    }
    while !m.d.is_empty() {
        let pos = m.d.len() - 1;
        if to_a[pos] { m.sa() } else { m.sb() }
    }
    while !m.a.is_empty() || !m.b.is_empty() {
        match (m.a.last(), m.b.last()) {
            (Some(x), Some(y)) => if x < y { m.ma() } else { m.mb() },
            (Some(_), None) => m.ma(),
            (None, Some(_)) => m.mb(),
            (None, None) => break,
        }
    }
}

fn main() {
    let n = 52usize;
    let cap = 40usize;

    // sample LDS trajectories
    println!("sample LDS trajectories (random n=52, bury_larger=true):");
    let mut rng = Rng::new(42);
    for _ in 0..6 {
        let p = rng.perm(n);
        let (passes, sorted, traj, mv) = run(&p, true, cap);
        let ok = sorted && check(&p, &mv);
        println!("  lds0={:>2} -> {:?}  ({passes} passes, sorted={sorted}, valid={ok}, moves={})", traj[0], traj, mv.len());
    }

    for &bury_larger in &[true, false] {
        println!("\n=== bury_larger={bury_larger} ===");
        let trials = 3000usize;
        let mut rng = Rng::new(7);
        let (mut sp, mut conv, mut smoves, mut shut, mut snat, mut shalve) = (0u64, 0usize, 0u64, 0u64, 0u64, 0u64);
        let mut verified = 0usize;
        for _ in 0..trials {
            let p = rng.perm(n);
            let (passes, sorted, traj, mv) = run(&p, bury_larger, cap);
            if sorted {
                conv += 1;
                assert!(check(&p, &mv));
                verified += 1;
                sp += passes as u64;
                smoves += mv.len() as u64;
            }
            shut += hutucker_cost(&p) as u64;
            snat += natural_cost(&p) as u64;
            // did pass 1 at least roughly halve LDS?
            if traj.len() >= 2 && (traj[1] as f64) <= 0.6 * traj[0] as f64 {
                shalve += 1;
            }
        }
        println!("random n=52 ({trials} decks; {verified} replay-verified sorted):");
        println!("  converged: {conv}/{trials};  pass1 cut LDS by >=40%: {shalve}/{trials}");
        if conv > 0 {
            println!(
                "  mean passes={:.2}  mean moves={:.1}  (natural-merge passes~log2(r)=5; log2(LDS)~3.8)",
                sp as f64 / conv as f64, smoves as f64 / conv as f64
            );
            println!(
                "  mean moves={:.1}  vs Hu-Tucker {:.1}  vs natural {:.1}",
                smoves as f64 / conv as f64, shut as f64 / trials as f64, snat as f64 / trials as f64
            );
        }
    }

    // LDS reduction per pass: aggregate ratio lds[k+1]/lds[k]
    println!("\nmean LDS after each pass (random n=52, bury_larger=true, 3000 decks):");
    let mut rng = Rng::new(99);
    let mut by_pass = vec![(0u64, 0usize); cap + 1];
    for _ in 0..3000 {
        let p = rng.perm(n);
        let (_p, _s, traj, _m) = run(&p, true, cap);
        for (k, &l) in traj.iter().enumerate() {
            by_pass[k].0 += l as u64;
            by_pass[k].1 += 1;
        }
    }
    for k in 0..8 {
        if by_pass[k].1 > 0 {
            println!("  after pass {k}: mean LDS = {:.2} ({} decks still active)", by_pass[k].0 as f64 / by_pass[k].1 as f64, by_pass[k].1);
        }
    }

    // The crux test: does ONE chain-parity pass (the recpat split) halve LDS on D?
    println!("\nchain-parity pass — single-pass LDS reduction (random n=52, 3000 decks):");
    let mut rng = Rng::new(123);
    let (mut before, mut after, mut halved, mut down1) = (0u64, 0u64, 0usize, 0usize);
    for _ in 0..3000 {
        let p = rng.perm(n);
        let l0 = lds(&p);
        let mut m = Emitter::new(&p);
        one_pass_chainparity(&mut m);
        assert_eq!(m.d.len(), n); // valid permutation back on D
        let l1 = lds(&m.d);
        before += l0 as u64;
        after += l1 as u64;
        if (l1 as f64) <= 0.6 * l0 as f64 { halved += 1; }
        if l1 + 1 >= l0 && l1 < l0 { down1 += 1; } // decreased by ~1 only
    }
    println!(
        "  mean LDS {:.2} -> {:.2}  (halved >=40%: {halved}/3000;  cut by ~1 only: {down1}/3000)",
        before as f64 / 3000.0, after as f64 / 3000.0
    );
    // iterate chain-parity passes to sort and count
    let mut rng = Rng::new(321);
    let (mut sp, mut conv) = (0u64, 0usize);
    for _ in 0..2000 {
        let p = rng.perm(n);
        let mut m = Emitter::new(&p);
        let mut passes = 0;
        while ascending_runs(&m.d).len() > 1 && passes < cap {
            one_pass_chainparity(&mut m);
            passes += 1;
        }
        if ascending_runs(&m.d).len() <= 1 {
            assert!(check(&p, &m.moves));
            conv += 1;
            sp += passes as u64;
        }
    }
    if conv > 0 {
        println!("  iterated chain-parity: converged {conv}/2000, mean passes={:.2} (log2(LDS)~3.8, log2(r)~4.7)", sp as f64 / conv as f64);
    } else {
        println!("  iterated chain-parity: did NOT converge within cap on most decks");
    }
}
