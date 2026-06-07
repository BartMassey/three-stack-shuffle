//! Harness for iterating deterministic sorters against the optimum (n<=14) and
//! the merge frontier (n=52). Candidate sorters live here (scratch) until one is
//! worth promoting to the library. Each is verified by replay.

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{base_len, Card, Move, State};
use splitmerge::planner::rollout;
use splitmerge::search::ida_star;
use splitmerge::sorters::{hutucker_cost, hutucker_sort};
use splitmerge::util::Rng;

/// A live machine that records its moves.
struct Mac {
    d: Vec<Card>,
    a: Vec<Card>,
    b: Vec<Card>,
    mv: Vec<Move>,
    n: usize,
}
impl Mac {
    fn new(deck: &[Card]) -> Self {
        Mac {
            d: deck.to_vec(),
            a: vec![],
            b: vec![],
            mv: vec![],
            n: deck.len(),
        }
    }
    fn sa(&mut self) {
        let c = self.d.pop().unwrap();
        self.a.push(c);
        self.mv.push(Move::SA);
    }
    fn sb(&mut self) {
        let c = self.d.pop().unwrap();
        self.b.push(c);
        self.mv.push(Move::SB);
    }
    fn ma(&mut self) {
        let c = self.a.pop().unwrap();
        self.d.push(c);
        self.mv.push(Move::MA);
    }
    fn mb(&mut self) {
        let c = self.b.pop().unwrap();
        self.d.push(c);
        self.mv.push(Move::MB);
    }
    fn pour_back(&mut self) {
        loop {
            let nxt = (base_len(&self.d) + 1) as Card;
            if nxt as usize > self.n {
                break;
            }
            if self.a.last() == Some(&nxt) {
                self.ma();
            } else if self.b.last() == Some(&nxt) {
                self.mb();
            } else {
                break;
            }
        }
    }
    /// Route the deck top onto a buffer by 2-pile patience with the given overflow
    /// rule. `to_larger`: on a forced bury, place on the larger top (least gap).
    fn route(&mut self, to_larger: bool) {
        let c = *self.d.last().unwrap();
        let ta = self.a.last().copied();
        let tb = self.b.last().copied();
        let a_ok = ta.is_some_and(|t| t > c);
        let b_ok = tb.is_some_and(|t| t > c);
        if a_ok && b_ok {
            if ta.unwrap() <= tb.unwrap() {
                self.sa()
            } else {
                self.sb()
            }
        } else if a_ok {
            self.sa()
        } else if b_ok {
            self.sb()
        } else if self.a.is_empty() {
            self.sa()
        } else if self.b.is_empty() {
            self.sb()
        } else if (ta.unwrap() >= tb.unwrap()) == to_larger {
            self.sa()
        } else {
            self.sb()
        }
    }
}

/// Clean two-phase: drain the whole deck into two patience piles, then merge with
/// bounce (lift each blocker above the target across to the other buffer).
fn drain_merge(deck: &[Card], to_larger: bool) -> Vec<Move> {
    let mut m = Mac::new(deck);
    m.pour_back();
    while base_len(&m.d) < m.n {
        // drain remaining above-base deck
        let k = base_len(&m.d);
        while m.d.len() > k {
            m.route(to_larger);
        }
        let target = (k + 1) as Card;
        if m.a.contains(&target) {
            while *m.a.last().unwrap() != target {
                m.ma();
                m.sb();
            }
            m.ma();
        } else {
            while *m.b.last().unwrap() != target {
                m.mb();
                m.sa();
            }
            m.mb();
        }
        m.pour_back();
    }
    m.mv
}

fn cost(name: &str, f: &dyn Fn(&[Card]) -> Vec<Move>, deck: &[Card]) -> usize {
    let mv = f(deck);
    debug_assert!(
        State::from_deck(deck.to_vec()).applied(&mv).is_goal(),
        "{name} fails"
    );
    mv.len()
}

/// Two-phase with a *forced* split colouring (`color[i]` = buffer for the i-th
/// departing card), then merge with bounce. `None` if it can't proceed.
fn drain_merge_colored(deck: &[Card], color: &[bool]) -> usize {
    let mut m = Mac::new(deck);
    let mut i = 0;
    while !m.d.is_empty() {
        if color[i] {
            m.sa()
        } else {
            m.sb()
        }
        i += 1;
    }
    while base_len(&m.d) < m.n {
        let target = (base_len(&m.d) + 1) as Card;
        if m.a.contains(&target) {
            while *m.a.last().unwrap() != target {
                m.ma();
                m.sb();
            }
            m.ma();
        } else {
            while *m.b.last().unwrap() != target {
                m.mb();
                m.sa();
            }
            m.mb();
        }
    }
    m.mv.len()
}

/// Min-bury 2-colouring of the departure sequence (max union of two decreasing
/// subsequences) by an O(n^3) DP over (index, top-of-A, top-of-B). Returns the
/// colouring in departure order (`true` = A). 0 = empty pile (always "good").
fn min_bury_coloring(dep: &[Card]) -> Vec<bool> {
    let n = dep.len();
    use std::collections::HashMap;
    let mut memo: HashMap<(usize, Card, Card), u32> = HashMap::new();
    fn f(
        i: usize,
        ta: Card,
        tb: Card,
        dep: &[Card],
        memo: &mut HashMap<(usize, Card, Card), u32>,
    ) -> u32 {
        if i == dep.len() {
            return 0;
        }
        if let Some(&v) = memo.get(&(i, ta, tb)) {
            return v;
        }
        let c = dep[i];
        let bad_a = (ta != 0 && ta < c) as u32;
        let bad_b = (tb != 0 && tb < c) as u32;
        let v = (bad_a + f(i + 1, c, tb, dep, memo)).min(bad_b + f(i + 1, ta, c, dep, memo));
        memo.insert((i, ta, tb), v);
        v
    }
    f(0, 0, 0, dep, &mut memo);
    // reconstruct
    let mut color = vec![false; n];
    let (mut ta, mut tb) = (0u8, 0u8);
    for i in 0..n {
        let c = dep[i];
        let bad_a = (ta != 0 && ta < c) as u32;
        let bad_b = (tb != 0 && tb < c) as u32;
        let va = bad_a + f(i + 1, c, tb, dep, &mut memo);
        let vb = bad_b + f(i + 1, ta, c, dep, &mut memo);
        if va <= vb {
            color[i] = true;
            ta = c;
        } else {
            tb = c;
        }
    }
    color
}

fn min_bury_sort(deck: &[Card]) -> usize {
    let dep: Vec<Card> = deck.iter().rev().copied().collect();
    let color = min_bury_coloring(&dep);
    drain_merge_colored(deck, &color)
}

/// Hill-climb the split colouring: from a start, flip single bits keeping any
/// improvement (cost via `drain_merge_colored`), with a few random restarts.
fn split_ls(deck: &[Card], restarts: usize) -> usize {
    let n = deck.len();
    let dep: Vec<Card> = deck.iter().rev().copied().collect();
    let mut rng = Rng::new(
        deck.iter()
            .map(|&x| x as u64)
            .sum::<u64>()
            .wrapping_mul(2654435761),
    );
    let mut global = usize::MAX;
    for r in 0..restarts {
        let mut color = if r == 0 {
            min_bury_coloring(&dep)
        } else {
            (0..n).map(|_| rng.f64() < 0.5).collect()
        };
        let mut best = drain_merge_colored(deck, &color);
        loop {
            let mut improved = false;
            for i in 0..n {
                color[i] = !color[i];
                let c = drain_merge_colored(deck, &color);
                if c < best {
                    best = c;
                    improved = true;
                } else {
                    color[i] = !color[i];
                }
            }
            if !improved {
                break;
            }
        }
        global = global.min(best);
    }
    global
}

/// Min cost over all 2^n split colourings (the best fixed two-phase sorter).
fn best_split(deck: &[Card]) -> usize {
    let n = deck.len();
    let mut best = usize::MAX;
    for mask in 0u32..(1 << n) {
        let color: Vec<bool> = (0..n).map(|i| (mask >> i) & 1 == 1).collect();
        best = best.min(drain_merge_colored(deck, &color));
    }
    best
}

/// Find the min-cost split colouring and report its structure.
fn dump_best_colorings() {
    println!("\n=== structure of the best split (departure order = deck top first) ===");
    let n = 10;
    let mut rng = Rng::new(123);
    for _ in 0..6 {
        let deck = rng.perm(n);
        let dep: Vec<Card> = deck.iter().rev().copied().collect(); // departure order
        let (mut best_cost, mut best_mask) = (usize::MAX, 0u32);
        for mask in 0u32..(1 << n) {
            let color: Vec<bool> = (0..n).map(|i| (mask >> i) & 1 == 1).collect();
            let c = drain_merge_colored(&deck, &color);
            if c < best_cost {
                best_cost = c;
                best_mask = mask;
            }
        }
        // color[i] is for the i-th departing card; build A/B departure subsequences
        let a: Vec<Card> = (0..n)
            .filter(|&i| (best_mask >> i) & 1 == 1)
            .map(|i| dep[i])
            .collect();
        let b: Vec<Card> = (0..n)
            .filter(|&i| (best_mask >> i) & 1 == 0)
            .map(|i| dep[i])
            .collect();
        let buries = |s: &[Card]| {
            s.iter()
                .enumerate()
                .filter(|&(i, &x)| s[..i].iter().any(|&y| y < x))
                .count()
        };
        println!("dep {dep:?}  cost={best_cost}");
        println!("   A(dep order) {a:?}  buries={}", buries(&a));
        println!("   B(dep order) {b:?}  buries={}", buries(&b));
    }
}

fn brute_force_experiment() {
    println!("\n=== best fixed two-phase split vs opt (brute force over 2^n colourings) ===");
    println!(" n   opt   best-2phase   min-bury-split   ratios");
    for n in [8usize, 10, 12, 14] {
        let mut rng = Rng::new(7 + n as u64);
        let (mut so, mut sb, mut sp, mut cnt) = (0.0, 0.0, 0.0, 0);
        for _ in 0..8 {
            let deck = rng.perm(n);
            let st = State::from_deck(deck.clone());
            let Some(o) = ida_star(&st, &h_joint, 20_000_000).0 else {
                continue;
            };
            so += o as f64;
            sb += best_split(&deck) as f64;
            sp += min_bury_sort(&deck) as f64;
            cnt += 1;
        }
        let c = cnt as f64;
        println!(
            " {n:2}  {:.1}    {:.1}          {:.1}              best={:.3} minbury={:.3}",
            so / c,
            sb / c,
            sp / c,
            sb / so,
            sp / so
        );
    }
}

fn split_ls_experiment() {
    println!("=== split hill-climb (split_ls) vs opt (n<=14) and merge (n=52) ===");
    for n in [10usize, 12, 14] {
        let mut rng = Rng::new(500 + n as u64);
        let (mut so, mut sl, mut cnt) = (0.0, 0.0, 0);
        for _ in 0..30 {
            let deck = rng.perm(n);
            let Some(o) = ida_star(&State::from_deck(deck.clone()), &h_joint, 20_000_000).0 else {
                continue;
            };
            so += o as f64;
            sl += split_ls(&deck, 12) as f64;
            cnt += 1;
        }
        println!(
            " n={n}: split_ls/opt = {:.3}  (opt {:.1}, split_ls {:.1})",
            sl / so,
            so / cnt as f64,
            sl / cnt as f64
        );
    }
    let mut rng = Rng::new(52);
    let (mut sm, mut sl) = (0.0, 0.0);
    let trials = 20;
    for _ in 0..trials {
        let deck = rng.perm(52);
        sm += hutucker_cost(&deck) as f64;
        sl += split_ls(&deck, 16) as f64;
    }
    println!(
        " n=52: merge {:.1}   split_ls {:.1}   (ratio {:.3})",
        sm / trials as f64,
        sl / trials as f64,
        sl / sm
    );
}

/// The "interleave" class: deck = [1, m+1, 2, m+2, ..., m, 2m] (riffle of the two
/// halves). Departure order is a union of two decreasing subsequences, so B=0 and
/// opt=2n (zero transfers) -- yet it has m = n/2 ascending runs, so the merge
/// sorter pays ~log(n/2) passes. The transfer view sees 0; the run view sees log.
fn obvious_class() {
    println!("interleave deck (B=0 in transfer view): opt vs merge");
    println!(" n |  opt  h_joint  2n | hutucker  merge/opt");
    for n in [8usize, 10, 12, 14, 16] {
        let m = n / 2;
        let mut deck: Vec<Card> = Vec::new();
        for i in 1..=m {
            deck.push(i as Card);
            deck.push((m + i) as Card);
        }
        let st = State::from_deck(deck.clone());
        let opt = ida_star(&st, &h_joint, 50_000_000).0;
        let hj = h_joint(&st);
        let ht = hutucker_cost(&deck);
        println!(
            " {n:2} |  {:?}   {hj:3}   {} | {ht:4}    {:.2}",
            opt,
            2 * n,
            ht as f64 / opt.unwrap_or(1) as f64
        );
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("obvious") {
        obvious_class();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("ls") {
        split_ls_experiment();
        return;
    }
    type Candidate = (&'static str, Box<dyn Fn(&[Card]) -> Vec<Move>>);
    let candidates: Vec<Candidate> = vec![
        ("merge", Box::new(|d: &[Card]| hutucker_sort(d))),
        (
            "rollout",
            Box::new(|d: &[Card]| rollout(&State::from_deck(d.to_vec()))),
        ),
        ("drain_merge", Box::new(|d: &[Card]| drain_merge(d, true))),
        (
            "min_bury",
            Box::new(|d: &[Card]| {
                let dep: Vec<Card> = d.iter().rev().copied().collect();
                let color = min_bury_coloring(&dep);
                // re-derive moves (drain_merge_colored returns cost; rebuild for replay)
                let mut m = Mac::new(d);
                let mut i = 0;
                while !m.d.is_empty() {
                    if color[i] {
                        m.sa()
                    } else {
                        m.sb()
                    }
                    i += 1;
                }
                while base_len(&m.d) < m.n {
                    let target = (base_len(&m.d) + 1) as Card;
                    if m.a.contains(&target) {
                        while *m.a.last().unwrap() != target {
                            m.ma();
                            m.sb();
                        }
                        m.ma();
                    } else {
                        while *m.b.last().unwrap() != target {
                            m.mb();
                            m.sa();
                        }
                        m.mb();
                    }
                }
                m.mv
            }),
        ),
    ];

    dump_best_colorings();
    brute_force_experiment();
    println!("=== cost / opt  (mean over random decks, n<=14) ===");
    print!(" n  ");
    for (name, _) in &candidates {
        print!("{name:>12}");
    }
    println!("    opt");
    for n in [10usize, 12, 14] {
        let mut rng = Rng::new(100 + n as u64);
        let mut sums = vec![0.0f64; candidates.len()];
        let mut opts = 0.0;
        let mut cnt = 0;
        for _ in 0..40 {
            let deck = rng.perm(n);
            let st = State::from_deck(deck.clone());
            let Some(o) = ida_star(&st, &h_joint, 20_000_000).0 else {
                continue;
            };
            for (i, (name, f)) in candidates.iter().enumerate() {
                sums[i] += cost(name, f, &deck) as f64 / o as f64;
            }
            opts += o as f64;
            cnt += 1;
        }
        print!("{n:2}  ");
        for s in &sums {
            print!("{:>12.3}", s / cnt as f64);
        }
        println!("   {:.1}", opts / cnt as f64);
    }

    println!("\n=== cost vs merge frontier (n=52, 30 decks) ===");
    let mut rng = Rng::new(52);
    let mut sums = vec![0.0f64; candidates.len()];
    for _ in 0..30 {
        let deck = rng.perm(52);
        for (i, (name, f)) in candidates.iter().enumerate() {
            sums[i] += cost(name, f, &deck) as f64;
        }
    }
    for (i, (name, _)) in candidates.iter().enumerate() {
        println!("  {name:12} {:.1}", sums[i] / 30.0);
    }
}
