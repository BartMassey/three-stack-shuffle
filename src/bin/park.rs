//! Scratch: is "D as transit only" (every MA/MB arrival lands on the pure base —
//! the transit-only *transfer reduction*; NOT the six-action machine, which allows
//! parking) WLOG-optimal, or does some deck force *parking* (an arrival onto a deck
//! that still holds unsettled cards)? If no length-opt solution is "faithful", the
//! transfer reduction is LOSSY for that deck. Checks exhaustively n<=8, samples n=9,10.

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{base_len, Card, Move, State};
use splitmerge::search::ida_star_path;
use splitmerge::util::{fxset, FxHashSet, Rng};

/// Does a length-`opt` solution exist where every MA/MB happens with D == pure
/// base (no unsettled card in D at arrival time)?
fn exists_faithful(start: &State, opt: u32) -> bool {
    let goal = State::goal(start.size());
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    fn dfs(
        s: &State,
        g: u32,
        bound: u32,
        last: Option<Move>,
        goal: &State,
        on_path: &mut FxHashSet<State>,
    ) -> bool {
        if s == goal {
            return g == bound;
        }
        if g + h_joint(s) > bound {
            return false;
        }
        for (t, mv) in s.successors() {
            if last == Some(mv.inv()) || on_path.contains(&t) {
                continue;
            }
            // arrival (MA/MB) only when D is pure base before the move
            if matches!(mv, Move::MA | Move::MB) && base_len(&s.d) != s.d.len() {
                continue;
            }
            on_path.insert(t.clone());
            let ok = dfs(&t, g + 1, bound, Some(mv), goal, on_path);
            on_path.remove(&t);
            if ok {
                return true;
            }
        }
        false
    }
    dfs(start, 0, opt, None, &goal, &mut on_path)
}

fn all_perms(n: usize) -> Vec<Vec<Card>> {
    let mut out = Vec::new();
    let mut p: Vec<Card> = (1..=n as Card).collect();
    fn rec(p: &mut Vec<Card>, i: usize, out: &mut Vec<Vec<Card>>) {
        if i == p.len() {
            out.push(p.clone());
            return;
        }
        for j in i..p.len() {
            p.swap(i, j);
            rec(p, i + 1, out);
            p.swap(i, j);
        }
    }
    rec(&mut p, 0, &mut out);
    out
}

fn opt_of(deck: &[Card]) -> u32 {
    let st = State::from_deck(deck.to_vec());
    ida_star_path(&st, &h_joint, 20_000_000).0.unwrap().len() as u32
}

fn main() {
    // a specific check: the n=10 deck whose optimum I traced as parking.
    let probe: Vec<Card> = vec![4, 7, 8, 3, 5, 2, 9, 10, 1, 6];
    let o = opt_of(&probe);
    println!(
        "probe {probe:?}: opt={o}  faithful optimum exists? {}",
        exists_faithful(&State::from_deck(probe.clone()), o)
    );

    for n in 6..=8usize {
        let mut lossy: Option<Vec<Card>> = None;
        let mut checked = 0;
        for deck in all_perms(n) {
            let o = opt_of(&deck);
            if !exists_faithful(&State::from_deck(deck.clone()), o) {
                lossy = Some(deck);
                break;
            }
            checked += 1;
        }
        match lossy {
            Some(d) => println!("n={n}: PARKING NECESSARY (model lossy), first: {d:?} opt={}", opt_of(&d)),
            None => println!("n={n}: all {checked} decks have a faithful optimum (transit-only WLOG)"),
        }
    }

    for n in [9usize, 10] {
        let mut rng = Rng::new(30 + n as u64);
        let trials = if n == 9 { 3000 } else { 2000 };
        let (mut lossy_cnt, mut first) = (0, None);
        for _ in 0..trials {
            let deck = rng.perm(n);
            let o = opt_of(&deck);
            if !exists_faithful(&State::from_deck(deck.clone()), o) {
                lossy_cnt += 1;
                if first.is_none() {
                    first = Some(deck.clone());
                }
            }
        }
        print!("n={n} ({trials} random): parking necessary in {lossy_cnt}");
        if let Some(d) = first {
            print!("  first: {d:?} opt={}", opt_of(&d));
        }
        println!();
    }
}
