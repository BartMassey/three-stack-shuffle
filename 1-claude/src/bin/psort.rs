//! Experiment (b): the value-fit TWO-OPEN-PILES (patience-2) sorter.
//!
//! Use the two stacks as two patience piles: each departing card is placed on the pile
//! it "fits under" (pile top > card, so it becomes the new min-on-top) — value-aware,
//! non-adjacent combination, NOT positional merge. This subsumes reversal (cards are
//! placed individually). Two piles sort in ONE pass + one merge (= 2n moves) iff the
//! pop-order needs <= 2 decreasing piles, i.e. iff LDS(deck) <= 2 (LIS of the pop order).
//! Otherwise it gets STUCK (a card exceeds both tops -> a 3rd pile is needed) and we fall
//! back to Hu-Tucker. Everything is replay-verified to actually sort.

use splitmerge::machine::{Card, Move, State};
use splitmerge::sorters::{ascending_runs, hutucker_cost, hutucker_sort};
use splitmerge::util::Rng;

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
    fn merge_all(&mut self) {
        // both piles are sorted ascending (min on top); merge everything back to D.
        while !self.a.is_empty() && !self.b.is_empty() {
            if self.a.last().unwrap() < self.b.last().unwrap() { self.ma(); } else { self.mb(); }
        }
        while !self.a.is_empty() { self.ma(); }
        while !self.b.is_empty() { self.mb(); }
    }
}

/// One pass of 2-pile patience + final merge. `None` if it gets stuck (needs a 3rd pile).
fn patience2_sort(deck: &[Card]) -> Option<Vec<Move>> {
    let mut m = Emitter::new(deck);
    while let Some(&c) = m.d.last() {
        let a_fits = m.a.last().map_or(false, |&t| t > c);
        let b_fits = m.b.last().map_or(false, |&t| t > c);
        match (a_fits, b_fits) {
            // best-fit: place on the smaller top that still fits (keeps the larger pile open)
            (true, true) => if m.a.last() <= m.b.last() { m.sa() } else { m.sb() },
            (true, false) => m.sa(),
            (false, true) => m.sb(),
            (false, false) => {
                // c exceeds both tops (or a pile is empty): start a run on an empty pile
                if m.a.is_empty() { m.sa() }
                else if m.b.is_empty() { m.sb() }
                else { return None; } // stuck: both piles nonempty and c > both tops
            }
        }
    }
    m.merge_all();
    Some(m.moves)
}

fn hybrid_sort(deck: &[Card]) -> Vec<Move> {
    patience2_sort(deck).unwrap_or_else(|| hutucker_sort(deck))
}

/// Longest strictly decreasing subsequence length (patience succeeds iff this <= 2).
fn lds(deck: &[Card]) -> usize {
    // = LIS of reversed/negated; simple O(n^2)
    let n = deck.len();
    let mut dp = vec![1usize; n];
    let mut best = if n == 0 { 0 } else { 1 };
    for i in 0..n {
        for j in 0..i {
            if deck[j] > deck[i] && dp[j] + 1 > dp[i] {
                dp[i] = dp[j] + 1;
            }
        }
        best = best.max(dp[i]);
    }
    best
}

fn check(deck: &[Card], moves: &[Move]) -> bool {
    State::from_deck(deck.to_vec()).applied(moves) == State::goal(deck.len())
}

fn report(label: &str, deck: &[Card]) {
    let hut = hutucker_cost(deck);
    let runs = ascending_runs(deck).len();
    match patience2_sort(deck) {
        Some(mv) => {
            assert!(check(deck, &mv), "patience2 did not sort {label}");
            println!(
                "{label:<22} patience2={:>4} (sorted ✓)   hutucker={hut:>4}   ratio={:.2}   asc_runs={runs} lds={}",
                mv.len(), mv.len() as f64 / hut.max(1) as f64, lds(deck)
            );
        }
        None => println!("{label:<22} patience2=STUCK (lds={}>2)        hutucker={hut:>4}   asc_runs={runs}", lds(deck)),
    }
}

fn main() {
    let n = 52usize;

    let sorted: Vec<Card> = (1..=n as Card).collect();
    report("sorted", &sorted);
    let reversed: Vec<Card> = (1..=n as Card).rev().collect();
    report("reversed", &reversed);
    let k = n / 2;
    let mut inter: Vec<Card> = Vec::new();
    for t in 0..k { inter.push(1 + t as Card); inter.push(1 + (k + t) as Card); }
    report("interleave(obvious)", &inter);
    // a 3-way interleave (LDS=3): should get stuck
    let mut three: Vec<Card> = Vec::new();
    let third = n / 3;
    for t in 0..third {
        three.push(1 + t as Card);
        three.push(1 + (third + t) as Card);
        three.push(1 + (2 * third + t) as Card);
    }
    while (three.len() as Card) < n as Card { let v = three.len() as Card + 1; three.push(v); }
    report("3-interleave", &three);

    // random: how often does patience2 apply, and hybrid vs Hu-Tucker?
    println!();
    let trials = 5000usize;
    let mut rng = Rng::new(7);
    let (mut succ, mut sum_hut, mut sum_hyb, mut sum_succ_p, mut sum_succ_h) = (0usize, 0u64, 0u64, 0u64, 0u64);
    let mut verified = 0usize;
    for _ in 0..trials {
        let p = rng.perm(n);
        let hut = hutucker_cost(&p) as u64;
        sum_hut += hut;
        let hyb = hybrid_sort(&p);
        assert!(check(&p, &hyb));
        verified += 1;
        sum_hyb += hyb.len() as u64;
        if let Some(pp) = patience2_sort(&p) {
            succ += 1;
            sum_succ_p += pp.len() as u64;
            sum_succ_h += hut;
        }
    }
    println!("random n=52 ({trials} decks, all hybrid outputs replay-verified: {verified}/{trials}):");
    println!("  patience2 applies (LDS<=2): {succ}/{trials} = {:.2}%", 100.0 * succ as f64 / trials as f64);
    if succ > 0 {
        println!("  on those: mean patience2={:.1} vs mean hutucker={:.1}", sum_succ_p as f64 / succ as f64, sum_succ_h as f64 / succ as f64);
    }
    println!("  mean hybrid={:.1}  vs mean hutucker={:.1}  (hybrid falls back to Hu-Tucker when stuck)",
        sum_hyb as f64 / trials as f64, sum_hut as f64 / trials as f64);

    // small n: patience applies more often (LDS<=2 is common when n small)
    println!("\npatience2 applicability vs n (random):");
    for nn in [4usize, 6, 8, 10, 14, 20, 30] {
        let mut r = Rng::new(100 + nn as u64);
        let (mut s, t) = (0usize, 4000usize);
        for _ in 0..t { if patience2_sort(&r.perm(nn)).is_some() { s += 1; } }
        println!("  n={nn:>2}: {:.1}% of decks", 100.0 * s as f64 / t as f64);
    }
}
