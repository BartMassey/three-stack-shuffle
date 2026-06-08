//! Experiment (c'), REAL recursive patience: the move-emitting, replay-verified
//! realization of the idealized `recpat` model. Settles the open lead in CURRENT.md:
//! does the idealized -24%-vs-merge survive realization, or does the non-positional
//! (interleaved) split inflate it back toward / past merge?
//!
//! The split is by chain-2-coloring (halve the LDS), NOT a positional run cut, so the
//! two groups are INTERLEAVED on D and must be routed apart — that routing is the tax
//! the merge sorter never pays (its runs are contiguous, so its split is free). The
//! recursion is realized by NESTED PARKING (work on a buffer's top while lower cards sit
//! inert) + MERGE-BY-EXACT-COUNT, exactly like `sorters::realize_tree`.
//!
//! Contract of `rec_sort(k)`: the top `k` of D become a single ascending run on top of
//! D; A and B are restored to their incoming heights (parked cards below untouched).
//!
//!   base  (LDS<=2): distribute by color onto the 2 buffers (each color = one increasing
//!                   chain = a sorted pile, min-on-top), merge from buffers.      2k
//!   recur (LDS>=3): distribute by color onto buffers (the interleaved split, k);
//!                   bring a back (MA), rec_sort it; bring b back above it (MB),
//!                   rec_sort it; merge the two now-CONTIGUOUS sorted runs.    k + 2k
//!
//! Bring-back (D->buf->D, two reversals) preserves vector order, so each half's LDS is
//! <= ceil(LDS/2) < LDS: the recursion strictly shrinks the LDS and terminates.

use splitmerge::machine::{Card, Move, State};
use splitmerge::sorters::hutucker_cost;
use splitmerge::util::Rng;

/// Longest strictly decreasing subsequence (vector order); == #cover-chains below.
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

/// Dilworth cover into increasing chains (greedy patience). Returns the chain index of
/// each element (vector order); #chains == LDS. A strictly-decreasing subsequence meets
/// each increasing chain at most once, so 2-coloring the chains halves the LDS.
fn cover_chains(s: &[Card]) -> (Vec<usize>, usize) {
    let mut tops: Vec<Card> = Vec::new();
    let mut idx = vec![0usize; s.len()];
    for (p, &c) in s.iter().enumerate() {
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

fn is_ascending(s: &[Card]) -> bool {
    s.windows(2).all(|w| w[0] < w[1])
}

/// A live machine that records its moves (top of each stack at the vector end).
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

    /// Pour the min-on-top pile of `na` cards (A's top) and `nb` cards (B's top) back to
    /// D as one ascending run, by exact count (never digs beneath the parked cards).
    fn merge_buffers(&mut self, mut na: usize, mut nb: usize) {
        while na > 0 && nb > 0 {
            if self.a.last().unwrap() < self.b.last().unwrap() {
                self.ma();
                na -= 1;
            } else {
                self.mb();
                nb -= 1;
            }
        }
        while na > 0 { self.ma(); na -= 1; }
        while nb > 0 { self.mb(); nb -= 1; }
    }

    /// Sort the top `k` of D into a single ascending run on top of D.
    fn rec_sort(&mut self, k: usize) {
        if k <= 1 {
            return;
        }
        let seg: Vec<Card> = self.d[self.d.len() - k..].to_vec(); // bottom-to-top
        if is_ascending(&seg) {
            return; // already a sorted run
        }
        let (idx, nch) = cover_chains(&seg);
        // value -> color (chain parity). seg values are distinct.
        let maxv = *seg.iter().max().unwrap() as usize;
        let mut color = vec![0u8; maxv + 1];
        for (p, &v) in seg.iter().enumerate() {
            color[v as usize] = (idx[p] % 2) as u8;
        }
        // distribute: pop top k of D, route each card to A (color 0) or B (color 1).
        let (mut ka, mut kb) = (0usize, 0usize);
        for _ in 0..k {
            let v = *self.d.last().unwrap();
            if color[v as usize] == 0 {
                self.sa();
                ka += 1;
            } else {
                self.sb();
                kb += 1;
            }
        }
        debug_assert_eq!(ka + kb, k);

        if nch <= 2 {
            // base: each color is one increasing chain, so A and B are sorted piles
            // (min on top). Merge them straight back. Total 2k.
            self.merge_buffers(ka, kb);
            return;
        }

        // recursive: groups aren't sorted yet. Both nonempty and each shorter than k.
        debug_assert!(ka > 0 && kb > 0 && ka < k && kb < k);
        // bring a (top ka of A) back to D, in restored vector order, and sort it.
        for _ in 0..ka {
            self.ma();
        }
        self.rec_sort(ka); // sorted-a now the top ka of D
                           // bring b (top kb of B) back above sorted-a, and sort it.
        for _ in 0..kb {
            self.mb();
        }
        self.rec_sort(kb); // sorted-b top kb of D, sorted-a the ka below it (contiguous)
                           // merge the two contiguous ascending runs: park b onto A, a onto B, pour.
        for _ in 0..kb {
            self.sa();
        }
        for _ in 0..ka {
            self.sb();
        }
        self.merge_buffers(kb, ka);
    }
}

/// Real recursive-patience sort: emit moves that sort `deck`, replay-verified by caller.
fn recreal_sort(deck: &[Card]) -> Vec<Move> {
    let mut m = Emitter::new(deck);
    let n = m.d.len();
    m.rec_sort(n);
    m.moves
}

// ---- idealized model (from recpat) for side-by-side comparison ------------------

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

fn rec_cost_ideal(s: &[Card]) -> usize {
    if s.len() <= 1 || is_ascending(s) {
        return 0;
    }
    if lds(s) <= 2 {
        return 2 * s.len();
    }
    let (a, b) = split_halving(s);
    2 * s.len() + rec_cost_ideal(&a) + rec_cost_ideal(&b)
}

fn check(deck: &[Card], moves: &[Move]) -> bool {
    State::from_deck(deck.to_vec()).applied(moves) == State::goal(deck.len())
}

fn report(label: &str, deck: &[Card]) {
    let mv = recreal_sort(deck);
    let ok = check(deck, &mv);
    println!(
        "{label:<22} real={:>4} (sorted {})  ideal={:>4}  hutucker={:>4}  | lds={} real/ideal={:.2} real/hut={:.2}",
        mv.len(),
        if ok { "✓" } else { "✗ FAIL" },
        rec_cost_ideal(deck),
        hutucker_cost(deck),
        lds(deck),
        mv.len() as f64 / rec_cost_ideal(deck).max(1) as f64,
        mv.len() as f64 / hutucker_cost(deck).max(1) as f64,
    );
    assert!(ok, "recreal failed to sort {label}");
}

fn main() {
    let n = 52usize;

    let sorted: Vec<Card> = (1..=n as Card).collect();
    report("sorted", &sorted);
    let reversed: Vec<Card> = (1..=n as Card).rev().collect();
    report("reversed", &reversed);
    let k = n / 2;
    let mut inter: Vec<Card> = Vec::new();
    for t in 0..k {
        inter.push(1 + t as Card);
        inter.push(1 + (k + t) as Card);
    }
    report("interleave(obvious)", &inter);

    // random average, every output replay-verified.
    println!();
    let trials = 5000usize;
    let mut rng = Rng::new(11);
    let (mut sreal, mut sideal, mut shut, mut slds) = (0u64, 0u64, 0u64, 0u64);
    let (mut real_beats_hut, mut verified) = (0usize, 0usize);
    for _ in 0..trials {
        let p = rng.perm(n);
        let mv = recreal_sort(&p);
        assert!(check(&p, &mv));
        verified += 1;
        let r = mv.len() as u64;
        let h = hutucker_cost(&p) as u64;
        sreal += r;
        sideal += rec_cost_ideal(&p) as u64;
        shut += h;
        slds += lds(&p) as u64;
        if r < h {
            real_beats_hut += 1;
        }
    }
    let m = |x: u64| x as f64 / trials as f64;
    println!("random n=52 ({trials} decks, all replay-verified: {verified}/{trials}):");
    println!(
        "  real={:.1}  ideal={:.1}  hutucker={:.1}  (mean LDS={:.1})",
        m(sreal),
        m(sideal),
        m(shut),
        m(slds)
    );
    println!(
        "  real vs ideal: +{:.1} ({:.1}% inflation)   real vs hutucker: {:+.1} ({:+.1}%), real wins {real_beats_hut}/{trials}",
        m(sreal) - m(sideal),
        100.0 * (m(sreal) - m(sideal)) / m(sideal),
        m(sreal) - m(shut),
        100.0 * (m(sreal) - m(shut)) / m(shut),
    );

    // scaling: does real beat / lose to Hu-Tucker as n grows?
    println!("\nreal vs ideal vs hutucker mean, by n (1500 decks each):");
    for nn in [16usize, 24, 32, 40, 52, 80, 120] {
        let mut rr = Rng::new(500 + nn as u64);
        let (mut a, mut b, mut c, t) = (0u64, 0u64, 0u64, 1500usize);
        for _ in 0..t {
            let p = rr.perm(nn);
            let mv = recreal_sort(&p);
            debug_assert!(check(&p, &mv));
            a += mv.len() as u64;
            b += rec_cost_ideal(&p) as u64;
            c += hutucker_cost(&p) as u64;
        }
        println!(
            "  n={nn:>3}: real={:.1}  ideal={:.1}  hutucker={:.1}  | real/hut={:.2}  real/ideal={:.2}",
            a as f64 / t as f64,
            b as f64 / t as f64,
            c as f64 / t as f64,
            a as f64 / c as f64,
            a as f64 / b as f64,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every permutation up to n=8 sorts correctly (the contract holds exhaustively).
    #[test]
    fn exhaustive_sorts_small_n() {
        fn permute(p: &mut Vec<Card>, i: usize, f: &mut dyn FnMut(&[Card])) {
            if i == p.len() {
                f(p);
                return;
            }
            for j in i..p.len() {
                p.swap(i, j);
                permute(p, i + 1, f);
                p.swap(i, j);
            }
        }
        for n in 1..=8usize {
            let mut p: Vec<Card> = (1..=n as Card).collect();
            permute(&mut p, 0, &mut |q| {
                assert!(check(q, &recreal_sort(q)), "recreal failed on {q:?}");
            });
        }
    }
}
