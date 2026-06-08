//! Scratch: find the smallest deck that FORCES a card to transfer >= 2 times
//! (enter D >= 3 times) in *every* optimal solution. A card's D-arrivals = its
//! transfers + 1 (the settle); >= 3 arrivals = transferred twice. "Forced" =
//! no optimal-length solution exists with every card arriving <= 2 times.
//!
//! NOTE: the n=9 search here is SAMPLED; `dbx9.rs` does it exhaustively. The true
//! minimum is `[3,5,2,4,7,9,6,8,1]` (opt 22), not the sampled `[6,2,3,5,8,9,1,7,4]`
//! (opt 24). dbx9/dbxshow also show a double can be forced WITHOUT re-burial
//! (the "hot-potato" mechanism — e.g. card 1, the global min, which can't be buried).

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{Card, Move, State};
use splitmerge::search::ida_star_path;
use splitmerge::util::{fxset, FxHashSet, Rng};

fn arrivals(start: &State, path: &[Move]) -> Vec<u32> {
    let n = start.size();
    let mut e = vec![0u32; n + 1];
    let mut s = start.clone();
    for &mv in path {
        s.apply(mv);
        if matches!(mv, Move::MA | Move::MB) {
            e[*s.d.last().unwrap() as usize] += 1;
        }
    }
    e
}

/// Does a length-`opt` solution exist with every card arriving <= 2 times?
fn exists_simple_optimal(start: &State, opt: u32) -> bool {
    let goal = State::goal(start.size());
    let mut entries = vec![0u32; start.size() + 1];
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    fn dfs(
        s: &State,
        g: u32,
        bound: u32,
        last: Option<Move>,
        goal: &State,
        entries: &mut [u32],
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
            let mut undo: Option<usize> = None;
            if matches!(mv, Move::MA | Move::MB) {
                let c = *t.d.last().unwrap() as usize;
                if entries[c] >= 2 {
                    continue; // would be the 3rd arrival -> not "simple"
                }
                entries[c] += 1;
                undo = Some(c);
            }
            on_path.insert(t.clone());
            let ok = dfs(&t, g + 1, bound, Some(mv), goal, entries, on_path);
            on_path.remove(&t);
            if let Some(c) = undo {
                entries[c] -= 1;
            }
            if ok {
                return true;
            }
        }
        false
    }
    dfs(start, 0, opt, None, &goal, &mut entries, &mut on_path)
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

fn report(deck: &[Card]) {
    let st = State::from_deck(deck.to_vec());
    let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
    let path = path.unwrap();
    let opt = path.len() as u32;
    let arr = arrivals(&st, &path);
    let maxarr = arr.iter().max().unwrap();
    let doublers: Vec<usize> = (1..arr.len()).filter(|&c| arr[c] >= 3).collect();
    let simple = exists_simple_optimal(&st, opt);
    let mut s = st.clone();
    let mut trace = String::new();
    for &mv in &path {
        s.apply(mv);
        trace.push(match mv {
            Move::SA => 'a',
            Move::SB => 'b',
            Move::MA => 'A',
            Move::MB => 'B',
        });
    }
    println!(
        "deck {deck:?}  opt={opt}  max_arrivals={maxarr}  double-transfer cards (arrivals>=3)={doublers:?}  arrivals={:?}",
        &arr[1..]
    );
    println!("   optimal trace (lower=split, UPPER=merge): {trace}");
    println!(
        "   simple optimal (all arrivals<=2) exists? {}  => double-transfer {}",
        simple,
        if simple { "AVOIDABLE" } else { "FORCED" }
    );
}

fn main() {
    // Exhaustive: smallest n and lex-first deck where the optimal trace shows a
    // double transfer, and (separately) where it is FORCED.
    for n in 6..=8usize {
        let mut first_occurs: Option<Vec<Card>> = None;
        let mut first_forced: Option<Vec<Card>> = None;
        for deck in all_perms(n) {
            let st = State::from_deck(deck.clone());
            let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
            let Some(path) = path else { continue };
            let arr = arrivals(&st, &path);
            if *arr.iter().max().unwrap() >= 3 {
                if first_occurs.is_none() {
                    first_occurs = Some(deck.clone());
                }
                let opt = path.len() as u32;
                if !exists_simple_optimal(&st, opt) {
                    first_forced = Some(deck.clone());
                    break;
                }
            }
        }
        println!("=== n={n} ===");
        match &first_occurs {
            Some(d) => {
                print!("  double OCCURS in an optimum, first: ");
                report(d);
            }
            None => println!("  no optimum exhibits a double transfer at n={n}"),
        }
        match &first_forced {
            Some(d) => {
                print!("  double FORCED, first: ");
                report(d);
            }
            None => println!("  no FORCED double transfer at n={n}"),
        }
        println!();
    }

    // n=9,10,11: sample (exhaustive too big), look for the first forced double.
    for n in [9usize, 10, 11] {
        let mut rng = Rng::new(20 + n as u64);
        let (mut occurs, mut forced, mut forced_deck) = (0, 0, None);
        let trials = if n == 9 { 4000 } else { 3000 };
        for _ in 0..trials {
            let deck = rng.perm(n);
            let st = State::from_deck(deck.clone());
            let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
            let Some(path) = path else { continue };
            let arr = arrivals(&st, &path);
            if *arr.iter().max().unwrap() >= 3 {
                occurs += 1;
                if !exists_simple_optimal(&st, path.len() as u32) {
                    forced += 1;
                    if forced_deck.is_none() {
                        forced_deck = Some(deck.clone());
                    }
                }
            }
        }
        println!("=== n={n} ({trials} random) ===  double occurs in {occurs}, forced in {forced}");
        if let Some(d) = forced_deck {
            print!("  first forced: ");
            report(&d);
        }
        println!();
    }
}
