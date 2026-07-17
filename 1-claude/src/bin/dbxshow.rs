//! Scratch: trace + explain specific forced-double decks (companion to dbx9).
//! Also finds, among ALL forced n=9 decks, the minimum-opt ones and shows a trace.

use splitmerge::heuristics::h_joint;
use splitmerge::machine::{base_len, Card, Move, State};
use splitmerge::search::ida_star_path;
use splitmerge::util::{fxset, FxHashSet};

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

fn exists_simple_optimal(start: &State, opt: u32) -> bool {
    let goal = State::goal(start.size());
    let mut entries = vec![0u32; start.size() + 1];
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    fn dfs(
        s: &State, g: u32, bound: u32, last: Option<Move>, goal: &State,
        entries: &mut [u32], on_path: &mut FxHashSet<State>,
    ) -> bool {
        if s == goal { return g == bound; }
        if g + h_joint(s) > bound { return false; }
        for (t, mv) in s.successors() {
            if last == Some(mv.inv()) || on_path.contains(&t) { continue; }
            let mut undo: Option<usize> = None;
            if matches!(mv, Move::MA | Move::MB) {
                let c = *t.d.last().unwrap() as usize;
                if entries[c] >= 2 { continue; }
                entries[c] += 1; undo = Some(c);
            }
            on_path.insert(t.clone());
            let ok = dfs(&t, g + 1, bound, Some(mv), goal, entries, on_path);
            on_path.remove(&t);
            if let Some(c) = undo { entries[c] -= 1; }
            if ok { return true; }
        }
        false
    }
    dfs(start, 0, opt, None, &goal, &mut entries, &mut on_path)
}

/// Does a length-`opt` solution exist in which card `cap_card` arrives <= 2 times
/// (all OTHER cards unconstrained)? If NO, then every optimum triple-arrives
/// `cap_card`. With cap_card = 1 (the global minimum, which can never land on a
/// smaller card), a NO answer means the forced double is *necessarily* hot-potato.
fn exists_optimal_sparing(start: &State, opt: u32, cap_card: usize) -> bool {
    let goal = State::goal(start.size());
    let mut on_path: FxHashSet<State> = fxset();
    on_path.insert(start.clone());
    fn dfs(
        s: &State, g: u32, bound: u32, last: Option<Move>, goal: &State,
        cap_card: usize, used: u32, on_path: &mut FxHashSet<State>,
    ) -> bool {
        if s == goal { return g == bound; }
        if g + h_joint(s) > bound { return false; }
        for (t, mv) in s.successors() {
            if last == Some(mv.inv()) || on_path.contains(&t) { continue; }
            let mut nused = used;
            if matches!(mv, Move::MA | Move::MB) && *t.d.last().unwrap() as usize == cap_card {
                nused += 1;
                if nused >= 3 { continue; } // would be card's 3rd arrival
            }
            on_path.insert(t.clone());
            let ok = dfs(&t, g + 1, bound, Some(mv), goal, cap_card, nused, on_path);
            on_path.remove(&t);
            if ok { return true; }
        }
        false
    }
    dfs(start, 0, opt, None, &goal, cap_card, 0, &mut on_path)
}

/// Verbose, state-annotated trace of one optimal solution.
fn explain(deck: &[Card]) {
    let st = State::from_deck(deck.to_vec());
    let (path, _) = ida_star_path(&st, &h_joint, 50_000_000);
    let path = path.unwrap();
    let opt = path.len() as u32;
    let arr = arrivals(&st, &path);
    let doublers: Vec<usize> = (1..arr.len()).filter(|&c| arr[c] >= 3).collect();
    let simple = exists_simple_optimal(&st, opt);
    println!("\n========== deck {deck:?}  (base_len={}) ==========", base_len(deck));
    println!("opt={opt}  arrivals(by card 1..n)={:?}  doublers={doublers:?}  forced={}",
        &arr[1..], !simple);
    let mut s = st.clone();
    println!("  step  move            D / A / B");
    println!("   --   start           {:?} / {:?} / {:?}", s.d, s.a, s.b);
    for (i, &mv) in path.iter().enumerate() {
        s.apply(mv);
        let (name, kind) = match mv {
            Move::SA => ("SA  D->A", "split"),
            Move::SB => ("SB  D->B", "split"),
            Move::MA => ("MA  A->D", "MERGE"),
            Move::MB => ("MB  B->D", "MERGE"),
        };
        // mark D-arrivals of the doubler cards
        let mut tag = String::new();
        if matches!(mv, Move::MA | Move::MB) {
            let c = *s.d.last().unwrap() as usize;
            if doublers.contains(&c) {
                tag = format!("   <- card {c} arrives at D (#{})", arr_upto(&st, &path[..=i], c));
            }
        }
        println!("  {:>3}   {name:<6} {kind:>5}  {:?} / {:?} / {:?}{tag}", i + 1, s.d, s.a, s.b);
    }
}

fn arr_upto(start: &State, path: &[Move], card: usize) -> u32 {
    let mut e = 0;
    let mut s = start.clone();
    for &mv in path {
        s.apply(mv);
        if matches!(mv, Move::MA | Move::MB) && *s.d.last().unwrap() as usize == card {
            e += 1;
        }
    }
    e
}

fn all_perms(n: usize) -> Vec<Vec<Card>> {
    let mut out = Vec::new();
    let mut p: Vec<Card> = (1..=n as Card).collect();
    fn rec(p: &mut Vec<Card>, i: usize, out: &mut Vec<Vec<Card>>) {
        if i == p.len() { out.push(p.clone()); return; }
        for j in i..p.len() { p.swap(i, j); rec(p, i + 1, out); p.swap(i, j); }
    }
    rec(&mut p, 0, &mut out);
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "minopt" {
        // Find the global minimum opt among all FORCED n=9 decks (single-thread,
        // slow-ish but fine), then explain the lex-first achiever.
        let mut best: Option<(u32, Vec<Card>)> = None;
        let mut count_at_best = 0usize;
        for deck in all_perms(9) {
            let st = State::from_deck(deck.clone());
            let (path, _) = ida_star_path(&st, &h_joint, 20_000_000);
            let Some(path) = path else { continue };
            let arr = arrivals(&st, &path);
            if *arr.iter().max().unwrap() < 3 { continue; }
            let opt = path.len() as u32;
            if exists_simple_optimal(&st, opt) { continue; }
            match &best {
                Some((b, _)) if opt > *b => {}
                Some((b, _)) if opt == *b => { count_at_best += 1; }
                _ => { best = Some((opt, deck.clone())); count_at_best = 1; }
            }
        }
        if let Some((b, d)) = &best {
            println!("min opt among forced n=9 decks = {b}; {count_at_best} forced decks at that opt; lex-first such = {d:?}");
        }
        return;
    }
    if args.len() > 1 && args[1] == "hotpotato" {
        // For each min-opt hot-potato deck (doubler = card 1), check whether ANY
        // optimum spares card 1 (keeps it <= 2 arrivals). NO => the forced double
        // is necessarily hot-potato (card 1 can never be re-buried).
        let decks: [&[Card]; 6] = [
            &[3, 5, 2, 4, 7, 9, 6, 8, 1],
            &[3, 5, 2, 4, 8, 9, 6, 7, 1],
            &[3, 6, 7, 2, 4, 8, 9, 1, 5],
            &[4, 5, 2, 3, 7, 9, 6, 8, 1],
            &[4, 5, 2, 3, 8, 9, 6, 7, 1],
            &[6, 7, 2, 3, 4, 8, 9, 1, 5],
        ];
        for d in decks {
            let st = State::from_deck(d.to_vec());
            let (path, _) = ida_star_path(&st, &h_joint, 50_000_000);
            let opt = path.unwrap().len() as u32;
            let spare = exists_optimal_sparing(&st, opt, 1);
            println!(
                "{:?}  opt={opt}  optimum sparing card 1 (<=2 arrivals)? {}  => card-1 double {}",
                d, spare, if spare { "AVOIDABLE (re-route possible)" } else { "FORCED (necessarily hot-potato)" }
            );
        }
        return;
    }
    if args.len() > 1 && args[1] == "dual" {
        // value-complement + reverse of the lex-first hot-potato deck
        let base = [3, 5, 2, 4, 7, 9, 6, 8, 1];
        let n = base.len() as Card;
        let dual: Vec<Card> = base.iter().rev().map(|&v| n + 1 - v).collect();
        println!("dual of {:?} = {:?}", base, dual);
        explain(&base);
        explain(&dual);
        return;
    }
    // Default: explain the lex-first forced deck and the recorded one.
    explain(&[3, 5, 2, 4, 7, 9, 6, 8, 1]); // exhaustive lex-first, opt 22
    explain(&[6, 2, 3, 5, 8, 9, 1, 7, 4]); // previously recorded witness, opt 24
}
